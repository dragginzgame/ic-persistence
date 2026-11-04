use super::*;
#[test]
#[ignore = "requires an explicitly prepared server URL and Memory fixture Wasm"]
fn installed_backing_custody() {
    let pic = crate::connect_to_server();
    qualify(&pic);
}
