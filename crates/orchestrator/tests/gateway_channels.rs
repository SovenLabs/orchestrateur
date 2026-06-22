//! Tests catalogue canaux gateway Phase 10 (feature `gateway`).

#![cfg(feature = "gateway")]

use orchestrator::ChannelCatalog;

#[test]
fn gateway_channel_catalog_has_at_least_fifteen_channels() {
    assert!(ChannelCatalog::new().count() >= 15);
}

#[test]
fn gateway_channel_catalog_has_eighteen_channels() {
    assert_eq!(ChannelCatalog::new().count(), 18);
}

#[test]
fn gateway_channel_catalog_includes_core_and_stub_channels() {
    let catalog = ChannelCatalog::new();
    for id in [
        "webchat",
        "webhook",
        "telegram",
        "discord",
        "slack",
        "whatsapp",
        "matrix",
        "nostr",
    ] {
        let channel = catalog
            .get(id)
            .unwrap_or_else(|| panic!("canal manquant: {id}"));
        assert!(!channel.display_name.is_empty());
    }
}

#[test]
fn dedicated_channels_are_flagged() {
    let catalog = ChannelCatalog::new();
    assert!(catalog.get("telegram").expect("telegram").dedicated);
    assert!(!catalog.get("whatsapp").expect("whatsapp").dedicated);
}