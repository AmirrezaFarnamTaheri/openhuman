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
    version: "0.19.2",
    release_url: "https://github.com/tinyhumansai/tinymcp/releases/tag/v0.19.2",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-rolling-arm64",
            archive: "tinymcp-0.19.2-archlinux-arm64.tar.gz",
            sha256: "cb357041fa7a3815b913fbf235bb10e9ff4c475e3fbf921bc93d8fff644e55e2",
        },
        PlatformAsset {
            host_key: "archlinux-rolling-x86_64",
            archive: "tinymcp-0.19.2-archlinux-x86_64.tar.gz",
            sha256: "90bc66c70fb1fb3581ca382e9281fc28bc27c836ec1f65998413a4e8641d4c7e",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinymcp-0.19.2-fedora-43-arm64.tar.gz",
            sha256: "cc805cc004f6baec0adc62e1153f0368ef17180e816c96a895db1b17726d4a2c",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinymcp-0.19.2-fedora-43-x86_64.tar.gz",
            sha256: "3ad7ff3b83922d5cf195dfdea9377ddc6295d7d523467f34f3449df38667bc3d",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinymcp-0.19.2-fedora-44-arm64.tar.gz",
            sha256: "d4a50e75598c3886527abf8f628d40f96b5a86488df3a59d9bafa1b317cf7576",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinymcp-0.19.2-fedora-44-x86_64.tar.gz",
            sha256: "62b0fd7a25128af5922ca1a9a80599b4b16fa53e8646c0ee5ff67babd1cf3cd3",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymcp-0.19.2-macos-15-arm64.tar.gz",
            sha256: "57cb298306eedc7c3149b1438b174fca2d73ce4bfe9ee68211887f9782c7a1c4",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymcp-0.19.2-macos-15-x86_64.tar.gz",
            sha256: "8e9f0cfb7886fbf462238257f056f9d3a2309a94328d79643d4283682648f4dc",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymcp-0.19.2-macos-26-arm64.tar.gz",
            sha256: "f566ef326b754d08b5579927a8df9c9c4db65cf0236468cdc520fce22163f843",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymcp-0.19.2-macos-26-x86_64.tar.gz",
            sha256: "37e3e974af8cdabc40483a6e137480a6d0cf9d3623bc98e7eba6f41ef77bbbf7",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymcp-0.19.2-ubuntu-22.04-arm64.tar.gz",
            sha256: "fe474bce2f4666d0e5c0ccfd6e582a33a9f170bbb8212e491c46fedc0ce5bbe3",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymcp-0.19.2-ubuntu-22.04-x86_64.tar.gz",
            sha256: "9a60527d63a162e4ee4a49b28cc9e52d996e32cebcf354107ec5671759888e92",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymcp-0.19.2-ubuntu-24.04-arm64.tar.gz",
            sha256: "33a9c5d5e1f54b1bd4bf6557695d59de414f7e07062bb052ec26807daa2b47e0",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymcp-0.19.2-ubuntu-24.04-x86_64.tar.gz",
            sha256: "699dc226b2c3c4a91f99e6f456f8a24ec14caeb77b2cd08b2741d9dccabb38a5",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymcp-0.19.2-windows-11-arm64.zip",
            sha256: "d1a067cf4e0bacec0645a949095124b86df6d967a0e01c3ddeb8ea1559becb49",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymcp-0.19.2-windows-2022-x86_64.zip",
            sha256: "ff01a8e99c9e793248e1479ff4ffd79d5c97283589bbf3ad7c0e48bbcde55a93",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymcp-0.19.2-windows-2025-x86_64.zip",
            sha256: "6a9f9c9c53e908d4868d7060502e4c5cd4ccc7ec6717df058aae14afdebe106e",
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
    version: "0.21.0",
    release_url: "https://github.com/tinyhumansai/tinyconnectors/releases/tag/v0.21.0",
    assets: &[
        PlatformAsset {
            host_key: "archlinux-arm64",
            archive: "tinyconnectors-0.21.0-archlinux-arm64.tar.gz",
            sha256: "d147513762e370f683b47f50c58ec9b68b7a62e19ed15759acc854c3e24da3ad",
        },
        PlatformAsset {
            host_key: "archlinux-x86_64",
            archive: "tinyconnectors-0.21.0-archlinux-x86_64.tar.gz",
            sha256: "1672f0299d80b7436dfb485ded0d0db8734482a62bae699179336b3267557ee4",
        },
        PlatformAsset {
            host_key: "fedora-43-arm64",
            archive: "tinyconnectors-0.21.0-fedora-43-arm64.tar.gz",
            sha256: "7993d561bc350d67ab60f8f5c31ed2705aca376fcd34c5a6c61cf127b63b76ac",
        },
        PlatformAsset {
            host_key: "fedora-43-x86_64",
            archive: "tinyconnectors-0.21.0-fedora-43-x86_64.tar.gz",
            sha256: "5fd3b94e49d0d45fb4e2a92a1a668a7a0d5cf8d3a08f6843c6381353f4209e5c",
        },
        PlatformAsset {
            host_key: "fedora-44-arm64",
            archive: "tinyconnectors-0.21.0-fedora-44-arm64.tar.gz",
            sha256: "003b09f5d71c170e72077f046fff429f8c8b61fe6031668b61fc736a22c37457",
        },
        PlatformAsset {
            host_key: "fedora-44-x86_64",
            archive: "tinyconnectors-0.21.0-fedora-44-x86_64.tar.gz",
            sha256: "d95a6bc885d78149bb5d26dce4b291a46b6ffd75d2ce8c6279ff36266448f7ce",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyconnectors-0.21.0-macos-15-arm64.tar.gz",
            sha256: "cf6135caf054d40a5af5b001cc9e549fbe63b1a05f65482d482025bc01cdf2a7",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyconnectors-0.21.0-macos-15-x86_64.tar.gz",
            sha256: "e8fff9402d4ba9453d7cf518e8bea94edae49da8709279e8aa78b432009e0116",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyconnectors-0.21.0-macos-26-arm64.tar.gz",
            sha256: "ca835cbb88e12de0678cc85be3a99cc6e42e415fa73a251c3b5afc344aee4a1e",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyconnectors-0.21.0-macos-26-x86_64.tar.gz",
            sha256: "59060a2f3c7aa71363f7c30b618d72b7181179459ba349f8ec0a9fa536919f44",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyconnectors-0.21.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "f9cc5b782436f3301fe06d62aed87c479ebd4c6745f0b26ea138825d5904f0f9",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyconnectors-0.21.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "f1210cc9ae65078d7f2604a4dc5dbe57a53b49a1e5017dee5d6e14e57b02de64",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyconnectors-0.21.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "99e8ef19397aa04684d1948b46d94eb94c5fa192ffbf3e47cbe6aedd1fd069b1",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyconnectors-0.21.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "cfb40dec770adb4a4b2d2db4161f3301f8a14fe8fa7dbb0683b6d579ff2ed2fe",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyconnectors-0.21.0-windows-11-arm64.zip",
            sha256: "49f26d501dce7bee9191f594d782cc4f5f7ff6d0ba9e710ab6413194fdc0c371",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyconnectors-0.21.0-windows-2022-x86_64.zip",
            sha256: "3e96e044e6a310c8e97c2c598198db79b26dca5cee597a7aeb51097d4be119ae",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyconnectors-0.21.0-windows-2025-x86_64.zip",
            sha256: "d7bfe7141ebedfe406db16fc0df0d93f6f658cb0cdbad0415b6c1f332c5167b7",
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
