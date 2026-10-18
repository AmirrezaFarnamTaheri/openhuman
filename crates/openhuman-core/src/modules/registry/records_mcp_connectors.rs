//! Registry records for the `tinymcp` and `tinyconnectors` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset, SourceSpec};

/// The `tinymcp` module: the Model Context Protocol client.
///
/// Owns both transports (Streamable HTTP and a subprocess over stdio), the
/// statically declared server set a host puts in its own configuration, the
/// dynamic registry of user-installed servers with its SQLite store, the
/// reconnect supervisor, the browser sign-in flow, and the write-audit log.
///
/// Lazy, because dialing an MCP server is something most sessions never do: a
/// host with no installed servers and no configured ones would otherwise pay a
/// download and a `dlopen` for a capability it never reaches. That differs from
/// the module's own `lazy = false` export hint, which speaks for a host whose
/// servers should be connected the moment it comes up — this host decides when
/// that moment is, and does so on the first ask.
///
/// **What stays out of the module is host policy**, and the split is the same
/// one the contract's own documentation draws: the prompt-injection scan over
/// remote tool definitions, the `mcp_clients` RPC surface, the
/// agent-facing tools, and the proxy *scoping* decision all belong to this
/// application's threat model, not to a protocol client. `tinymcp-bus` carries
/// the vocabulary; this table says which bytes may speak it.
pub(crate) const TINYMCP: ModuleRecord = ModuleRecord {
    id: "tinymcp",
    description: "Model Context Protocol client: transports, registry, and the write-audit log",
    bus_name: tinymcp_bus::names::INTERFACE,
    object_path: tinymcp_bus::names::OBJECT_PATH,
    version: "0.19.1",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.19.1",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-rolling-arm64",
            archive: "tinymcp-0.19.1-archlinux-arm64.tar.gz",
            sha256: "2164c5fe979fb9931c9aec0e6202be5a016a5b046882fd8190c942ee0ba056d1",
        },
        PlatformAsset {
            host_key: "archlinux-rolling-x86_64",
            archive: "tinymcp-0.19.1-archlinux-x86_64.tar.gz",
            sha256: "35f154f2aa2baea39277cfb100265d59f606f7b00273506e444b2b1726ed6e49",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinymcp-0.19.1-fedora-43-arm64.tar.gz",
            sha256: "816a23293925d71acd737f6125de11cd8eb6cffdb7da5dd0bedbec8e4b86eeef",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinymcp-0.19.1-fedora-43-x86_64.tar.gz",
            sha256: "dbb3277172f616af98c0f1fb50442a3a572df53adbfbb6c4bc22aa00958af548",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinymcp-0.19.1-fedora-44-arm64.tar.gz",
            sha256: "0f20b057bda562fd41239fe31178c812a56792687ad1443f6b6cbd33c4c8cf4e",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinymcp-0.19.1-fedora-44-x86_64.tar.gz",
            sha256: "81ced19858bc1a9a7fedcb6cf75069942809989ccc7d67839981cb56c170a3f6",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.19.1-macos-15-arm64.tar.gz",
            sha256: "a3a06500f3842e6cc153a13528b6ce3376bf7d0ec37736277fe0ba6cf01176cc",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.19.1-macos-15-x86_64.tar.gz",
            sha256: "b769fa83ddd5daf2b39cf5dc9cea93673e4d63745b8d8013a4440bb1a376593b",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.19.1-macos-26-arm64.tar.gz",
            sha256: "eacea8fa57470df2dce7c123cc33cd26006edf2089a50a6483fb450bdd891e79",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.19.1-macos-26-x86_64.tar.gz",
            sha256: "2b567495f1190b2935967b0a61750b461a31548613cff6f3bbb2f7c946e57447",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.19.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "4d3a856b23d229ec4d32cc81e05f3dadcb8830113f72e7c51eefeaa9e4b58314",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.19.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "d5fea2aba4dd93aa7f7875fd56f221355ac7de9b4c520bc18f41a67dcc53e56c",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.19.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "2f51e05a75acebacbb31b4a7b35ebe128b751224b5cc9550e1b192b7debad46b",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.19.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "d401450ff2f2d1990c2d7fa3fc5cc8a57a81f15555b03a6be0246095400186d9",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.19.1-windows-11-arm64.zip",
            sha256: "efd15c7efdb5d2e12d6f864c825e57bc0f3365330a528c47824d415bbeaf570c",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.19.1-windows-2022-x86_64.zip",
            sha256: "aec8f610d741912be2abadea88e5a013267ad7a86ad52e94234d705b24797617",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.19.1-windows-2025-x86_64.zip",
            sha256: "cfc07583c420e80e87a18ece75e4c472f7f8bf429ec79484925f37ded91f22b5",
        },
    ],
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinymcp",
        manifest: "crates/tinymcp/Cargo.toml",
        package: "tinymcp",
        lib_name: "tinymcp",
        features: &[],
    },
};

pub(crate) const TINYCONNECTORS: ModuleRecord = ModuleRecord {
    id: "tinyconnectors",
    description: "OAuth connector integrations: accounts, actions, and triggers",
    bus_name: tinyconnectors_bus::names::INTERFACE,
    object_path: tinyconnectors_bus::names::OBJECT_PATH,
    version: "0.19.3",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.19.3",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinyconnectors-0.19.3-archlinux-arm64.tar.gz",
            sha256: "1b223604c366769a67d0345d12da9b1b679722b54062157c68fba08b608f3ec9",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinyconnectors-0.19.3-archlinux-x86_64.tar.gz",
            sha256: "b7c83dbf1a4f40a343fe687708412f5b33c137cf384ebea5d8b9b6223d890f58",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinyconnectors-0.19.3-fedora-43-arm64.tar.gz",
            sha256: "386552936cc9964ec4a006664656fa1ac58aaccd104dbf8fd71626824805dac4",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinyconnectors-0.19.3-fedora-43-x86_64.tar.gz",
            sha256: "7d810750a28d64ff332448095c91e57064da1dfdd70028ac378c86c88df8ca2c",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinyconnectors-0.19.3-fedora-44-arm64.tar.gz",
            sha256: "a7094a70e28f99dbe4312be4c0d5b49337b5b35e605bd07c7dde323e247fcdf6",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinyconnectors-0.19.3-fedora-44-x86_64.tar.gz",
            sha256: "39c4938e9daba8643910e2b4f8573c0559d1cf8a15b44bd99e615a9dab926081",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.19.3-macos-15-arm64.tar.gz",
            sha256: "644a1c58c2ab7911ea2808c23b75b96e3ae312af1d12f97cca55f71971f551b9",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.19.3-macos-15-x86_64.tar.gz",
            sha256: "53297c0fffad47ac19dba05a9e360af1601cf4de779e7db0d9e06d2059e91099",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.19.3-macos-26-arm64.tar.gz",
            sha256: "96290a9f9b9b07fffbe3f333478002d7f46e5c2a563542d73ddac47420e597a4",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.19.3-macos-26-x86_64.tar.gz",
            sha256: "990a0414fb4a2fc834efeedcd5e4e51e7057aed928f9c51aa4fae2c4a43a7f0d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.19.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "f5ab8ced9095150df3ab0f3b84b4fe505a61d1fec64de535355d59a7ee13954f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.19.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "26daaa208a624c1afc5520fdd3d715eca032b956324ad5918431171097381620",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.19.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "ded63f30313e3196c0f75e6c6d211920d799df5e39b4d0657c4901de419db96e",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.19.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "15cfa5718aeabe2882503c618e22c5067f050fbc3aaa4be31e541071700900f5",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.19.3-windows-11-arm64.zip",
            sha256: "02a7e82e452c9190db68b5f2eef49b6fad73bf067dbb8f13ab3688c003dedf61",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.19.3-windows-2022-x86_64.zip",
            sha256: "cf0d99f72732d934b24428cfa2ef414a649d41c612de330ea76af4e037641822",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.19.3-windows-2025-x86_64.zip",
            sha256: "271bd3855a1565fedd041a02bbca2555d418bcd16d764b55385f6896d8e9682b",
        },
    ],
    // Lazy: a user with no connected accounts should not pay to load it, and
    // most sessions never touch a connector. Safe even signed out — the module
    // loads without configuration and still answers the capability members.
    load: LoadPolicy::Lazy,
    source: SourceSpec {
        submodule: "tinyconnectors",
        manifest: "crates/tinyconnectors/Cargo.toml",
        package: "tinyconnectors",
        lib_name: "tinyconnectors",
        features: &[],
    },
};
