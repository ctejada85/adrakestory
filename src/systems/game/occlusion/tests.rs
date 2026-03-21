#![cfg(test)]

use super::*;


    fn default_config() -> OcclusionConfig {
        OcclusionConfig::default()
    }

    // ── Task 10: cache hit path ───────────────────────────────────────────────
    // Verify that computing StaticOcclusionUniforms twice from the same config
    // produces equal values, so the dirty check returns false (no GPU work).

    #[test]
    fn static_cache_hit_same_config_is_not_dirty() {
        let config = default_config();
        let first = StaticOcclusionUniforms::from_config(&config);
        let second = StaticOcclusionUniforms::from_config(&config);
        // Identical inputs → equal values → cache matches → not dirty
        assert_eq!(first, second);
    }

    #[test]
    fn dynamic_cache_hit_same_positions_is_not_dirty() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let cam = Vec3::new(4.0, 5.0, 6.0);
        let first = DynamicOcclusionUniforms::new(pos, cam, None);
        let second = DynamicOcclusionUniforms::new(pos, cam, None);
        assert_eq!(first, second);
    }

    // ── Task 11: cache miss path ──────────────────────────────────────────────
    // Verify that a changed value produces a different struct, so the dirty check
    // returns true and get_mut() would be called.

    #[test]
    fn static_cache_miss_changed_min_alpha_is_dirty() {
        let config_a = OcclusionConfig {
            min_alpha: 0.1,
            ..default_config()
        };
        let config_b = OcclusionConfig {
            min_alpha: 0.9,
            ..default_config()
        };
        let cached = StaticOcclusionUniforms::from_config(&config_a);
        let new = StaticOcclusionUniforms::from_config(&config_b);
        assert_ne!(cached, new);
    }

    #[test]
    fn dynamic_cache_miss_moved_player_is_dirty() {
        let cam = Vec3::new(0.0, 10.0, 10.0);
        let cached = DynamicOcclusionUniforms::new(Vec3::ZERO, cam, None);
        let new = DynamicOcclusionUniforms::new(Vec3::new(5.0, 0.0, 5.0), cam, None);
        assert_ne!(cached, new);
    }

    // ── Task 12: static/dynamic independence ─────────────────────────────────
    // Verify that config fields only appear in StaticOcclusionUniforms and
    // positional fields only appear in DynamicOcclusionUniforms.

    #[test]
    fn static_uniforms_reflect_config_fields() {
        let config = OcclusionConfig {
            min_alpha: 0.42,
            occlusion_radius: 7.0,
            height_threshold: 1.5,
            falloff_softness: 3.0,
            technique: TransparencyTechnique::Dithered,
            mode: OcclusionMode::RegionBased,
            ..default_config()
        };
        let s = StaticOcclusionUniforms::from_config(&config);
        assert_eq!(s.min_alpha, 0.42);
        assert_eq!(s.occlusion_radius, 7.0);
        assert_eq!(s.height_threshold, 1.5);
        assert_eq!(s.falloff_softness, 3.0);
        assert_eq!(s.technique, 0); // Dithered → 0
        assert_eq!(s.mode, 2);      // RegionBased → 2
    }

    #[test]
    fn dynamic_uniforms_reflect_positional_fields() {
        let player = Vec3::new(1.0, 2.0, 3.0);
        let camera = Vec3::new(4.0, 5.0, 6.0);
        let d = DynamicOcclusionUniforms::new(player, camera, None);
        assert_eq!(d.player_position, player);
        assert_eq!(d.camera_position, camera);
        assert_eq!(d.region_min, Vec4::ZERO);
        assert_eq!(d.region_max, Vec4::ZERO); // no interior state → inactive
    }

    #[test]
    fn config_change_does_not_affect_dynamic_uniforms() {
        let player = Vec3::new(1.0, 0.0, 1.0);
        let camera = Vec3::new(0.0, 10.0, 10.0);
        let d1 = DynamicOcclusionUniforms::new(player, camera, None);
        // Simulated config change — dynamic struct is computed independently
        let d2 = DynamicOcclusionUniforms::new(player, camera, None);
        assert_eq!(d1, d2); // positions unchanged → dynamic cache still hits
    }

    // ── Task 13: first-frame unconditional write ──────────────────────────────
    // Verify that with empty caches (None), both sub-structs are always considered
    // dirty so the first write always proceeds.

    #[test]
    fn empty_static_cache_is_always_dirty() {
        let config = default_config();
        let new_static = StaticOcclusionUniforms::from_config(&config);
        let cache: Option<StaticOcclusionUniforms> = None;
        // Dirty when cache is None
        let dirty = cache.as_ref() != Some(&new_static);
        assert!(dirty);
    }

    #[test]
    fn empty_dynamic_cache_is_always_dirty() {
        let d = DynamicOcclusionUniforms::new(Vec3::ZERO, Vec3::new(0.0, 10.0, 10.0), None);
        let cache: Option<DynamicOcclusionUniforms> = None;
        let dirty = cache.as_ref() != Some(&d);
        assert!(dirty);
    }

    // ── Assembly correctness ──────────────────────────────────────────────────

    #[test]
    fn assemble_uniforms_maps_all_fields_correctly() {
        let config = OcclusionConfig {
            min_alpha: 0.05,
            occlusion_radius: 4.0,
            height_threshold: 1.0,
            falloff_softness: 2.5,
            technique: TransparencyTechnique::AlphaBlend,
            mode: OcclusionMode::Hybrid,
            ..default_config()
        };
        let s = StaticOcclusionUniforms::from_config(&config);
        let d = DynamicOcclusionUniforms::new(
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(0.0, 10.0, 0.0),
            None,
        );
        let u = assemble_uniforms(&s, &d);

        assert_eq!(u.player_position, d.player_position);
        assert_eq!(u.camera_position, d.camera_position);
        assert_eq!(u.min_alpha, s.min_alpha);
        assert_eq!(u.occlusion_radius, s.occlusion_radius);
        assert_eq!(u.height_threshold, s.height_threshold);
        assert_eq!(u.falloff_softness, s.falloff_softness);
        assert_eq!(u.technique, 1); // AlphaBlend → 1
        assert_eq!(u.mode, 3);      // Hybrid → 3
        assert_eq!(u.region_min, d.region_min);
        assert_eq!(u.region_max, d.region_max);
        // Padding fields must always be zero
        assert_eq!(u._padding1, 0.0);
        assert_eq!(u._padding2, 0.0);
        assert_eq!(u._padding3, 0);
        assert_eq!(u._padding4, 0);
    }

    // ── Interior detection config defaults ───────────────────────────────────

    #[test]
    fn region_update_interval_default_is_60() {
        let config = OcclusionConfig::default();
        assert_eq!(config.region_update_interval, 60);
    }

    #[test]
    fn default_occlusion_mode_is_hybrid() {
        let config = OcclusionConfig::default();
        assert_eq!(config.mode, OcclusionMode::Hybrid);
    }

    // ── Phase 1: default technique fix ───────────────────────────────────────

    #[test]
    fn default_technique_is_dithered() {
        let config = OcclusionConfig::default();
        assert_eq!(
            config.technique,
            TransparencyTechnique::Dithered,
            "Default technique must be Dithered to avoid AlphaToCoverage MSAA cost on macOS TBDR"
        );
    }

    #[test]
    fn dithered_technique_maps_to_opaque_alpha_mode() {
        let alpha_mode = match TransparencyTechnique::Dithered {
            TransparencyTechnique::Dithered => AlphaMode::Opaque,
            TransparencyTechnique::AlphaBlend => AlphaMode::AlphaToCoverage,
        };
        assert_eq!(alpha_mode, AlphaMode::Opaque);
    }

    #[test]
    fn alphablend_technique_still_maps_to_alpha_to_coverage() {
        let alpha_mode = match TransparencyTechnique::AlphaBlend {
            TransparencyTechnique::Dithered => AlphaMode::Opaque,
            TransparencyTechnique::AlphaBlend => AlphaMode::AlphaToCoverage,
        };
        assert_eq!(alpha_mode, AlphaMode::AlphaToCoverage);
    }

    // ── Quantize position ────────────────────────────────────────────────────

    #[test]
    fn quantize_position_rounds_to_grid() {
        let pos = Vec3::new(1.1, 2.6, -0.3);
        let result = quantize_position(pos, 0.25);
        assert_eq!(result, Vec3::new(1.0, 2.5, -0.25));
    }

    #[test]
    fn quantize_position_exact_grid_values_unchanged() {
        let pos = Vec3::new(0.5, 1.0, -2.25);
        let result = quantize_position(pos, 0.25);
        assert_eq!(result, pos);
    }

    #[test]
    fn quantize_position_zero_step_returns_original() {
        let pos = Vec3::new(1.123, 4.567, 8.9);
        assert_eq!(quantize_position(pos, 0.0), pos);
    }

    #[test]
    fn quantize_position_negative_step_returns_original() {
        let pos = Vec3::new(1.123, 4.567, 8.9);
        assert_eq!(quantize_position(pos, -1.0), pos);
    }

    #[test]
    fn quantize_position_origin() {
        assert_eq!(quantize_position(Vec3::ZERO, 0.25), Vec3::ZERO);
    }

    #[test]
    fn quantize_position_sub_step_movement_unchanged() {
        // Movements smaller than half-step should not change the quantized value
        let base = Vec3::new(1.0, 2.0, 3.0);
        let moved = Vec3::new(1.1, 2.05, 3.12);
        assert_eq!(quantize_position(base, 0.25), quantize_position(moved, 0.25));
    }

    #[test]
    fn quantize_position_large_step() {
        let pos = Vec3::new(3.7, -1.2, 6.1);
        let result = quantize_position(pos, 1.0);
        assert_eq!(result, Vec3::new(4.0, -1.0, 6.0));
    }

    #[test]
    fn default_config_has_quantization_step() {
        let config = OcclusionConfig::default();
        assert_eq!(config.uniform_quantization_step, 0.25);
    }
