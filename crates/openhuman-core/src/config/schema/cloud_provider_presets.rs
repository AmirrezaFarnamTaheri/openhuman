use super::*;

pub const EXTRA_CLOUD_PROVIDERS: &[BuiltinCloudProvider] = &[
    // 9Router is a user-managed local OpenAI-compatible inference gateway.
    // The gateway owns its own upstream accounts and separate API-key policy.
    BuiltinCloudProvider {
        slug: "9router",
        label: "9Router (Local Gateway)",
        endpoint: "http://127.0.0.1:20128/v1",
        auth_style: AuthStyle::Bearer,
    },
    // 9Router peer-derived API-key-only upstream presets. Non-key sessions
    // still require their own native credential/admission protocols.
    BuiltinCloudProvider {
        slug: "hyperbolic",
        label: "Hyperbolic",
        endpoint: "https://api.hyperbolic.xyz/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "chutes",
        label: "Chutes AI",
        endpoint: "https://llm.chutes.ai/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "featherless",
        label: "Featherless AI",
        endpoint: "https://api.featherless.ai/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "poolside",
        label: "Poolside",
        endpoint: "https://inference.poolside.ai/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "morph",
        label: "Morph",
        endpoint: "https://api.morphllm.com/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "perplexity",
        label: "Perplexity API",
        endpoint: "https://api.perplexity.ai",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "tencent-hunyuan",
        label: "Tencent Hunyuan",
        endpoint: "https://api.hunyuan.cloud.tencent.com/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "xiaomi-mimo",
        label: "Xiaomi MiMo (API Key)",
        endpoint: "https://api.xiaomimimo.com/v1",
        auth_style: AuthStyle::Bearer,
    },
    // Key-only OpenAI-compatible peers with documented public base URLs.
    // Model availability and per-model transport features are not implied.
    BuiltinCloudProvider {
        slug: "cohere",
        label: "Cohere",
        endpoint: "https://api.cohere.ai/compatibility/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "tokenrouter",
        label: "TokenRouter",
        endpoint: "https://api.tokenrouter.io/v1",
        auth_style: AuthStyle::Bearer,
    },
    // Region-scoped ModelArk access; AP and EU keys are not interchangeable.
    BuiltinCloudProvider {
        slug: "byteplus-modelark-ap",
        label: "BytePlus ModelArk (Asia Pacific)",
        endpoint: "https://ark.ap-southeast.bytepluses.com/api/v3",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "byteplus-modelark-eu",
        label: "BytePlus ModelArk (Europe)",
        endpoint: "https://ark.eu-west.bytepluses.com/api/v3",
        auth_style: AuthStyle::Bearer,
    },
    // Opt-in loopback gateway; OpenHuman never forwards Codex OAuth credentials.
    BuiltinCloudProvider {
        slug: "prodex",
        label: "Prodex Gateway",
        endpoint: "http://127.0.0.1:4000/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "llmtr",
        label: "LLMTR",
        endpoint: "https://llmtr.com/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "tokendance",
        label: "TokenDance",
        endpoint: "https://tokendance.space/gateway/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "siliconflow",
        label: "SiliconFlow",
        endpoint: "https://api.siliconflow.cn/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "qwen-pay-as-you-go",
        label: "Qwen Pay-As-You-Go",
        endpoint: "https://dashscope.aliyuncs.com/compatible-mode/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "qwen-token-plan",
        label: "Qwen Token Plan",
        endpoint: "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1",
        auth_style: AuthStyle::Bearer,
    },
    // Verified direct OpenAI-compatible provider endpoints; capability and
    // free-tier eligibility remain model/account-specific and must be discovered.
    BuiltinCloudProvider {
        slug: "aionlabs",
        label: "Aion Labs",
        endpoint: "https://api.aionlabs.ai/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "sambanova",
        label: "SambaNova",
        endpoint: "https://api.sambanova.ai/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "ovhcloud",
        label: "OVHcloud AI Endpoints",
        endpoint: "https://oai.endpoints.kepler.ai.cloud.ovh.net/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "nebius",
        label: "Nebius Token Factory",
        endpoint: "https://api.tokenfactory.nebius.com/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "llm7",
        label: "LLM7.io",
        endpoint: "https://api.llm7.io/v1",
        auth_style: AuthStyle::Bearer,
    },
    BuiltinCloudProvider {
        slug: "synthetic",
        label: "Synthetic",
        endpoint: "https://api.synthetic.new/openai/v1",
        auth_style: AuthStyle::Bearer,
    },
];
