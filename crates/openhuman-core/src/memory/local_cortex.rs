
/*!
 * Opt-in managed CortexDB for local desktop sessions.
 *
 * CortexDB currently publishes Linux servers and Linux Docker images, but
 * no native macOS/Windows server binaries. Run the pinned image via Docker
 * Desktop/Engine only when the user requests local memory. The app owns the
 * container, a persistent named volume and the credential: no copied endpoint
 * or manually created service.
 *
 * Docker is deliberately optional. This module is never called during app
 * startup, so users who use hosted memory do not pay for its downloads or RAM.
 */
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::{sleep, timeout};

use crate::config::Config;

use super::engine;
use super::error::{MemoryError, MemoryResult};

const IMAGE: &str = "cortexdb/cortexdb:v0.10.2";
const CONTAINER: &str = "openhuman-cortexdb";
pub const MANAGED_ENDPOINT: &str = "http://127.0.0.1:3141";

fn launch_args(key: &str) -> Vec<String> {
    [
        "run",
        "--detach",
        "--name",
        CONTAINER,
        "--restart",
        "unless-stopped",
        "--publish",
        "127.0.0.1:3141:3141",
        "--volume",
        "openhuman-cortexdb-data:/data",
        "--env",
    ]
    .into_iter()
    .map(str::to_string)
    .chain([format!("CORTEX_API_KEY={key}"), IMAGE.to_string()])
    .collect()
}

/// Recover the key of a previously provisioned container. Never write these
/// environment variables to logs: they may include other credentials.
fn recover_key(json: &str) -> Option<String> {
    serde_json::from_str::<Vec<String>>(json)
        .ok()?
        .into_iter()
        .find_map(|var| {
            var.strip_prefix("CORTEX_API_KEY=")
                .filter(|key| !key.trim().is_empty())
                .map(str::to_string)
        })
}

fn docker_executable() -> PathBuf {
    // Tauri launched from Finder often has no Homebrew/ Docker Desktop bin
    // directory on PATH; check the standard desktop installation locations.
    let candidates = [
        "/opt/homebrew/bin/docker",
        "/usr/local/bin/docker",
        "/Applications/Docker.app/Contents/Resources/bin/docker",
        "C:\\Program Files\\Docker\\Docker\\resources\\bin\\docker.exe",
    ];
    candidates
        .iter()
        .map(Path::new)
        .find(|path| path.is_file())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("docker"))
}

async fn docker(docker: &Path, args: &[&str], seconds: u64) -> MemoryResult<Output> {
    timeout(
        Duration::from_secs(seconds),
        Command::new(docker)
            .args(args)
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| MemoryError::Unavailable("Docker took too long to respond".to_string()))?
    .map_err(|_| {
        MemoryError::Unavailable(
            "Docker Desktop or Docker Engine is required for automatic local memory".to_string(),
        )
    })
}

async fn checked(docker_bin: &Path, args: &[&str], seconds: u64) -> MemoryResult<Output> {
    let output = docker(docker_bin, args, seconds).await?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(MemoryError::Unavailable(
            "Local memory could not start. Check Docker and make sure port 3141 is free."
                .to_string(),
        ))
    }
}

async fn wait_until_healthy(key: &str) -> MemoryResult<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|_| MemoryError::Unavailable("Could not check local memory".to_string()))?;
    for _ in 0..45 {
        if let Ok(response) = client
            .get(format!("{MANAGED_ENDPOINT}/v1/admin/health"))
            .bearer_auth(key)
            .send()
            .await
        {
            if response.status().is_success() {
                return Ok(());
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    Err(MemoryError::Unavailable(
        "Local memory started, but its health check did not become ready. Retry from Memory settings."
            .to_string(),
    ))
}

/// Ensure the app-owned server is running and return its credential. Called
/// only by an explicit RPC, never while binding memory on ordinary turns.
/// Existing containers are reused without deleting storage, even if the app
/// or Docker has been restarted.
pub async fn ensure_running(config: &Config) -> MemoryResult<String> {
    if crate::core::runtime::is_saas() {
        return Err(MemoryError::Unsupported(
            "Local memory can only be started on your own desktop".to_string(),
        ));
    }

    let docker_bin = docker_executable();
    checked(&docker_bin, &["info", "--format", "{{.ServerVersion}}"], 15).await?;
    let inspect = docker(
        &docker_bin,
        &["container", "inspect", "--format", "{{.State.Running}}", CONTAINER],
        15,
    )
    .await?;

    let key = if inspect.status.success() {
        // Refuse unrelated containers with this name; never replace/delete one.
        let image = checked(
            &docker_bin,
            &["container", "inspect", "--format", "{{.Config.Image}}", CONTAINER],
            15,
        )
        .await?;
        if !String::from_utf8_lossy(&image.stdout).trim().starts_with("cortexdb/cortexdb:") {
            return Err(MemoryError::Unavailable(
                "Another container already uses the local memory name".to_string(),
            ));
        }
        let env = checked(
            &docker_bin,
            &["container", "inspect", "--format", "{{json .Config.Env}}", CONTAINER],
            15,
        )
        .await?;
        let key = recover_key(&String::from_utf8_lossy(&env.stdout)).ok_or_else(|| {
            MemoryError::Unavailable(
                "The existing local memory container is missing its credential".to_string(),
            )
        })?;
        if String::from_utf8_lossy(&inspect.stdout).trim() != "true" {
            checked(&docker_bin, &["start", CONTAINER], 60).await?;
        }
        key
    } else {
        // Preserve a credential already configured for CortexDB. Using a new
        // password here would silently break switching back to a cloud endpoint.
        let key = engine::read_cortexdb_key(config)
            .map_err(|_| MemoryError::Unavailable("Could not read the memory credential".to_string()))?
            .unwrap_or_else(|| format!("oh-{}", uuid::Uuid::new_v4().simple()));
        let args = launch_args(&key);
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        checked(&docker_bin, &refs, 300).await?;
        key
    };

    wait_until_healthy(&key).await?;
    Ok(key)
}

#[cfg(test)]
#[path = "local_cortex_tests.rs"]
mod tests;
