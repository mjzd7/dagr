use dagr_cloud::{BlueGreenIndexManager, IndexSlot};
use dagr_core::Result;

#[test]
fn test_blue_green_index_cutover() -> Result<()> {
    let manager = BlueGreenIndexManager::new("v1.0.0");
    assert_eq!(manager.get_active_slot(), IndexSlot::Blue);
    assert_eq!(manager.get_active_version(), "v1.0.0");

    let shadow_slot = manager.prepare_shadow_index("v2.0.0-shadow");
    assert_eq!(shadow_slot, IndexSlot::Green);

    // Active remains Blue while shadow builds
    assert_eq!(manager.get_active_slot(), IndexSlot::Blue);

    // Cutover to Green
    manager.atomic_cutover(IndexSlot::Green)?;
    assert_eq!(manager.get_active_slot(), IndexSlot::Green);
    assert_eq!(manager.get_active_version(), "v2.0.0-shadow");

    Ok(())
}
