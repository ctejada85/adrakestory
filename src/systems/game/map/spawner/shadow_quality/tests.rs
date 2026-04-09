use super::super::shadow_params_for_quality;
use crate::systems::game::occlusion::ShadowQuality;

#[test]
fn none_disables_shadows() {
    let (shadows_enabled, _) = shadow_params_for_quality(ShadowQuality::None);
    assert!(!shadows_enabled);
}

#[test]
fn low_enables_shadows() {
    let (shadows_enabled, _) = shadow_params_for_quality(ShadowQuality::Low);
    assert!(shadows_enabled);
}

#[test]
fn high_enables_shadows() {
    let (shadows_enabled, _) = shadow_params_for_quality(ShadowQuality::High);
    assert!(shadows_enabled);
}

#[test]
fn characters_only_enables_shadows() {
    // CharactersOnly keeps the shadow map alive for character meshes.
    let (shadows_enabled, _) = shadow_params_for_quality(ShadowQuality::CharactersOnly);
    assert!(shadows_enabled);
}

#[test]
fn shadow_quality_default_is_low() {
    assert_eq!(ShadowQuality::default(), ShadowQuality::Low);
}
