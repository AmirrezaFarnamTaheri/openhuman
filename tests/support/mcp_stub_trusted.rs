//! Test-only declaration of the Cargo-built `test-mcp-stub` as an
//! operator-trusted stdio server.
//!
//! Registry-installed servers (`mcp.json`, the registry RPCs) are never
//! operator-trusted, so tinymcp requires a TinyBox jail for them. Registry
//! lifecycle tests use the loopback fixture in `mcp_http.rs` on every platform;
//! `mcp_registry_e2e.rs` separately verifies trusted stdio and sandbox refusal.
//!
//! The trusted stub is still needed only where a test is not on the registry
//! path: it declares the Cargo-built stub in the operator-owned `config.toml`
//! (`[[mcp_client.servers]]`, `operator_trusted = true`), which tinymcp starts
//! as an ordinary child process without a jail.

#![allow(dead_code)]

/// A `config.toml` fragment declaring the stub as an operator-trusted stdio
/// server called `name`, to append to a test config.
pub fn trusted_stub_toml(name: &str, stub: &str) -> String {
    format!(
        "\n[[mcp_client.servers]]\nname = {name:?}\ncommand = {stub:?}\noperator_trusted = true\n"
    )
}
