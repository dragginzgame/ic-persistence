#[test]
#[ignore = "requires an explicitly prepared server URL and Storage fixture Wasm"]
fn installed_storage_adapters() {
    let pic = crate::connect_to_server();
    super::qualify(&pic);
}
