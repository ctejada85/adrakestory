//! Runtime shadow quality application system.
//!
//! Watches `OcclusionConfig` for changes and applies the `ShadowQuality` setting to
//! the live scene — updating the `DirectionalLight`, `CascadeShadowConfig`, and
//! `NotShadowCaster` components on all `VoxelChunk` entities.
//!
//! Also fires on `Added<VoxelChunk>` so hot-reload re-spawns always get the correct
//! shadow state without a settings-menu round-trip.

use super::{shadow_params_for_quality, VoxelChunk};
use crate::systems::game::occlusion::{OcclusionConfig, ShadowQuality};
use bevy::pbr::{CascadeShadowConfig, NotShadowCaster};
use bevy::prelude::*;

/// Applies `OcclusionConfig::shadow_quality` to the live scene every time the
/// setting changes or new `VoxelChunk` entities appear.
///
/// Registered in `GameSystemSet::Visual` — runs after movement/physics, before camera.
pub fn apply_shadow_quality_system(
    config: Res<OcclusionConfig>,
    new_chunks: Query<Entity, Added<VoxelChunk>>,
    all_chunks: Query<Entity, With<VoxelChunk>>,
    mut dir_lights: Query<(&mut DirectionalLight, &mut CascadeShadowConfig)>,
    mut commands: Commands,
) {
    let setting_changed = config.is_changed();
    let has_new_chunks = !new_chunks.is_empty();

    if !setting_changed && !has_new_chunks {
        return;
    }

    // Update DirectionalLight when the setting changed (light survives hot-reload).
    if setting_changed {
        let (shadows_on, cascade_cfg) = shadow_params_for_quality(config.shadow_quality);
        for (mut light, mut cascade) in dir_lights.iter_mut() {
            light.shadows_enabled = shadows_on;
            *cascade = cascade_cfg.clone();
        }
    }

    // Determine which chunks to update: all on setting change, only new on hot-reload.
    let chunks_to_update: Vec<Entity> = if setting_changed {
        all_chunks.iter().collect()
    } else {
        new_chunks.iter().collect()
    };

    let needs_no_cast = config.shadow_quality == ShadowQuality::CharactersOnly;
    for entity in chunks_to_update {
        if needs_no_cast {
            commands.entity(entity).insert(NotShadowCaster);
        } else {
            commands.entity(entity).remove::<NotShadowCaster>();
        }
    }
}

#[cfg(test)]
mod tests {
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
}
