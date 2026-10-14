//! TTS provider implementations: cloud, local Piper, and external (slug-keyed).

use async_trait::async_trait;
use log::debug;

use super::super::local_speech::{synthesize_piper, PiperOptions};
use super::super::reply_speech::{synthesize_reply, ReplySpeechOptions};
use super::traits::TtsProvider;
use crate::config::Config;
use crate::core::Outcome;
use tinyinference_voice::external_tts::{ExternalTtsClient, TtsApiStyle};
use tinyinference_voice::reply::ReplySpeech as ReplySpeechResult;

const LOG_PREFIX: &str = "[voice-factory]";

// ---------------------------------------------------------------------------
// Cloud TTS
// ---------------------------------------------------------------------------

/// Cloud TTS — wraps [`synthesize_reply`] (backend ElevenLabs proxy).
pub struct CloudTtsProvider {
    voice: Option<String>,
}

impl CloudTtsProvider {
    pub fn new(voice: Option<String>) -> Self {
        Self { voice }
    }
}

#[async_trait]
impl TtsProvider for CloudTtsProvider {
    fn name(&self) -> &'static str {
        "cloud"
    }

    async fn synthesize(
        &self,
        config: &Config,
        text: &str,
        voice: Option<&str>,
    ) -> Result<Outcome<ReplySpeechResult>, String> {
        let resolved_voice = voice
            .map(str::to_string)
            .or_else(|| self.voice.clone())
            .filter(|s| !s.trim().is_empty());
        debug!(
            "{LOG_PREFIX} cloud TTS dispatch voice={} chars={}",
            resolved_voice.as_deref().unwrap_or("<default>"),
            text.len()
        );
        let opts = ReplySpeechOptions {
            voice_id: resolved_voice,
            model_id: None,
            output_format: None,
            voice_settings: None,
        };
        synthesize_reply(config, text, &opts).await
    }

    #[cfg(test)]
    fn configured_voice(&self) -> Option<&str> {
        self.voice.as_deref()
    }
}

// ---------------------------------------------------------------------------
// Local Piper TTS
// ---------------------------------------------------------------------------

/// Local Piper TTS — wraps [`synthesize_piper`].
pub struct PiperTtsProvider {
    voice: String,
}

impl PiperTtsProvider {
    pub fn new(voice: impl Into<String>) -> Self {
        Self {
            voice: voice.into(),
        }
    }
}

#[async_trait]
impl TtsProvider for PiperTtsProvider {
    fn name(&self) -> &'static str {
        "piper"
    }

    async fn synthesize(
        &self,
        config: &Config,
        text: &str,
        voice: Option<&str>,
    ) -> Result<Outcome<ReplySpeechResult>, String> {
        let resolved_voice = voice
            .map(str::to_string)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| self.voice.clone());
        debug!(
            "{LOG_PREFIX} piper TTS dispatch voice={} chars={}",
            resolved_voice,
            text.len()
        );
        let opts = PiperOptions {
            voice: Some(resolved_voice),
        };
        synthesize_piper(config, text, &opts).await
    }

    #[cfg(test)]
    fn configured_voice(&self) -> Option<&str> {
        Some(&self.voice)
    }
}

// ---------------------------------------------------------------------------
// External TTS provider (slug-keyed, third-party API)
// ---------------------------------------------------------------------------

/// Third-party TTS provider dispatched via the voice provider registry.
/// Supports OpenAI-compatible and ElevenLabs API styles; the HTTP clients
/// live in `tinyinference_voice::external_tts`.
pub struct ExternalTtsProvider {
    slug: String,
    default_voice: String,
    api_style: TtsApiStyle,
    client: ExternalTtsClient,
    // Only used when an explicitly configured native TTS model is present.
    // The default path remains owned by tinyinference_voice's existing client.
    native_model: Option<String>,
    native_endpoint: String,
    native_key: String,
    native_http: reqwest::Client,
}

const MAX_TTS_AUDIO_BYTES: usize = 16 * 1024 * 1024;

fn safe_native_tts_endpoint(raw: &str) -> Result<reqwest::Url, String> {
    let mut url = reqwest::Url::parse(raw)
        .map_err(|_| "[voice-tts] invalid native TTS server URL".to_string())?;
    if url.username() != "" || url.password().is_some() ||
       url.query().is_some() || url.fragment().is_some() {
        return Err("[voice-tts] native TTS URL must not contain credentials, query or fragment".into());
    }
    let host = url.host_str().unwrap_or_default();
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    let loopback = bare.eq_ignore_ascii_case("localhost") ||
        bare.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback());
    if url.scheme() != "https" && !(url.scheme() == "http" && loopback) {
        return Err("[voice-tts] native TTS requires HTTPS or loopback HTTP".into());
    }
    let path = url.path().trim_end_matches('/');
    if !path.ends_with("/v1") {
        return Err("[voice-tts] native TTS base URL must end in /v1".into());
    }
    url.set_path(&format!("{path}/"));
    Ok(url)
}

impl ExternalTtsProvider {
    pub fn new(
        slug: impl Into<String>,
        default_voice: impl Into<String>,
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
        api_style: TtsApiStyle,
    ) -> Self {
        let native_endpoint = endpoint.into();
        let native_key = api_key.into();
        Self {
            slug: slug.into(),
            default_voice: default_voice.into(),
            api_style,
            client: ExternalTtsClient::new(
                reqwest::Client::new(),
                native_endpoint.clone(),
                native_key.clone(),
                api_style,
            ),
            native_model: None,
            native_endpoint,
            native_key,
            native_http: reqwest::Client::new(),
        }
    }

    /// Opt-in native model selection for Kokoro, MLX Audio or other compatible
    /// TTS servers. Existing OpenAI/ElevenLabs clients retain their behavior.
    pub fn with_native_tts_model(mut self, model: &str) -> anyhow::Result<Self> {
        let model = model.trim();
        if self.api_style != TtsApiStyle::OpenaiAudio ||
            model.is_empty() || model.len() > 128 ||
            model.chars().any(char::is_control) {
            anyhow::bail!("native TTS model must be a nonempty OpenAI Audio model id");
        }
        safe_native_tts_endpoint(&self.native_endpoint)
            .map_err(anyhow::Error::msg)?;
        self.native_http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        self.native_model = Some(model.to_owned());
        Ok(self)
    }

    async fn synthesize_native_model(
        &self,
        text: &str,
        voice: &str,
        model: &str,
    ) -> Result<(Vec<u8>, String), String> {
        let root = safe_native_tts_endpoint(&self.native_endpoint)?;
        let url = root.join("audio/speech")
            .map_err(|_| "[voice-tts] invalid audio endpoint".to_string())?;
        let body = serde_json::json!({
            "model": model,
            "input": text,
            "voice": voice,
            "response_format": "wav"
        });
        let mut request = self.native_http.post(url).json(&body);
        if !self.native_key.trim().is_empty() {
            request = request.bearer_auth(&self.native_key);
        }
        let mut response = request.send().await
            .map_err(|_| "[voice-tts] native audio server unavailable or timed out".to_string())?;
        if !response.status().is_success() {
            return Err(format!("[voice-tts] native audio server HTTP {}", response.status()));
        }
        if response.content_length().is_some_and(|len| len > MAX_TTS_AUDIO_BYTES as u64) {
            return Err("[voice-tts] audio response exceeds size limit".into());
        }
        let mime = response.headers().get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("audio/wav")
            .to_string();
        if !mime.starts_with("audio/") && !mime.starts_with("application/octet-stream") {
            return Err("[voice-tts] native server response is not audio".into());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await
            .map_err(|_| "[voice-tts] could not read native audio response".to_string())?
        {
            if bytes.len().saturating_add(chunk.len()) > MAX_TTS_AUDIO_BYTES {
                return Err("[voice-tts] audio response exceeds size limit".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        if bytes.is_empty() {
            return Err("[voice-tts] native audio response is empty".into());
        }
        Ok((bytes, mime))
    }
}

#[async_trait]
impl TtsProvider for ExternalTtsProvider {
    fn name(&self) -> &'static str {
        "external"
    }

    async fn synthesize(
        &self,
        _config: &Config,
        text: &str,
        voice: Option<&str>,
    ) -> Result<Outcome<ReplySpeechResult>, String> {
        let resolved_voice = voice
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&self.default_voice);

        debug!(
            "{LOG_PREFIX} external TTS dispatch slug={} voice={} style={:?} chars={}",
            self.slug,
            resolved_voice,
            self.api_style,
            text.len()
        );

        let (audio_bytes, audio_mime) = match self.native_model.as_deref() {
            Some(model) => self.synthesize_native_model(text, resolved_voice, model).await?,
            None => self.client.synthesize(text, resolved_voice).await?,
        };

        use base64::Engine;
        let audio_base64 = base64::engine::general_purpose::STANDARD.encode(&audio_bytes);

        Ok(Outcome::single_log(
            ReplySpeechResult {
                audio_base64,
                audio_mime,
                visemes: Vec::new(),
                alignment: None,
            },
            format!("voice-factory: external TTS completed via {}", self.slug),
        ))
    }

    #[cfg(test)]
    fn configured_voice(&self) -> Option<&str> {
        Some(&self.default_voice)
    }
}


#[cfg(test)]
mod native_tts_model_tests {
    use super::*;

    #[test]
    fn native_audio_egress_is_https_or_loopback_only() {
        for url in [
            "https://server.example/v1",
            "http://localhost:8880/v1",
            "http://127.0.0.1:8880/v1",
            "http://[::1]:8880/v1",
        ] {
            assert!(safe_native_tts_endpoint(url).is_ok(), "{url}");
        }
        for url in [
            "http://server.example/v1",
            "http://127.0.0.1.evil.example/v1",
            "https://user:password@server.example/v1",
            "http://127.0.0.1:8880/v1?token=private",
            "file:///v1",
            "https://server.example/v2",
        ] {
            assert!(safe_native_tts_endpoint(url).is_err(), "{url}");
        }
    }

    #[test]
    fn native_audio_model_is_opt_in_and_rejects_bad_ids() {
        let base = || ExternalTtsProvider::new(
            "kokoro", "af_bella", "http://127.0.0.1:8880/v1",
            "", TtsApiStyle::OpenaiAudio
        );
        assert!(base().with_native_tts_model("kokoro").is_ok());
        assert!(base().with_native_tts_model("  ").is_err());
        assert!(base().with_native_tts_model("bad\nid").is_err());
        let remote = ExternalTtsProvider::new(
            "kokoro", "af_bella", "http://example.com/v1",
            "", TtsApiStyle::OpenaiAudio
        );
        assert!(remote.with_native_tts_model("kokoro").is_err());
        let eleven = ExternalTtsProvider::new(
            "elevenlabs", "voice", "https://api.elevenlabs.io/v1",
            "token", TtsApiStyle::ElevenLabs
        );
        assert!(eleven.with_native_tts_model("kokoro").is_err());
    }
}
