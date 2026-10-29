// The live voice handler and its disabled-feature stand-in share the same path.
#[cfg(all(feature = "voice", feature = "http-server"))]
use super::*;

#[cfg(all(feature = "voice", feature = "http-server"))]
#[test]
fn the_live_voice_socket_handler_is_the_core() {
    let _live = voice::live::ws::handle_live_voice_ws;
}

// The desktop port picker must cross embed's intentionally curated __host
// facade: the RPC server cannot reach openhuman-core directly.
#[test]
fn desktop_port_picker_is_exposed_to_rpc_host() {
    let _picker = super::platform::connectivity::rpc::pick_desktop_listen_port_for_host;
}
