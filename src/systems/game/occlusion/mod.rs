//! Voxel occlusion transparency system using extended materials.
//!
//! This module provides an extension to StandardMaterial that makes voxels transparent
//! when they block the camera's view of the player character. The transparency
//! is calculated per-pixel in the fragment shader for smooth, high-quality results.
//!
//! Uses Bevy's ExtendedMaterial to inherit all StandardMaterial features (PBR, shadows, etc.)
//! while adding custom occlusion transparency.
//!
//! ## Usage
//!
//! 1. Add `OcclusionPlugin` to your app
//! 2. Use `OcclusionMaterial` (ExtendedMaterial<StandardMaterial, OcclusionExtension>) for voxel chunks
//! 3. Store the material handle in `OcclusionMaterialHandle` resource
//! 4. The system automatically updates uniforms each frame
//!
//! ## Configuration
//!
//! Adjust `OcclusionConfig` resource to tweak the effect:
//! - `min_alpha`: Minimum transparency (0.0 = invisible, 1.0 = opaque)
//! - `occlusion_radius`: Horizontal radius of the occlusion zone
//! - `height_threshold`: Only affect voxels this far above the player
//! - `falloff_softness`: Smoothness of the vertical transition
//! - `technique`: Dithered (screen-door, default — no MSAA cost) or AlphaBlend (smooth, configurable via settings menu)
//! - `mode`: ShaderBased, RegionBased, or Hybrid occlusion mode

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};
use serde::{Deserialize, Serialize};

use super::components::{GameCamera, Player};
use super::interior_detection::InteriorState;
use crate::diagnostics::FrameProfiler;
use crate::profile_scope;

/// Transparency technique for occlusion effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TransparencyTechnique {
    /// Dithered transparency using screen-door effect (Bayer matrix pattern)
    /// Pros: No alpha sorting issues, works with opaque pipeline
    /// Cons: Visible dot pattern, especially at low alpha values
    #[default]
    Dithered,
    /// True alpha blending (smooth transparency like Photoshop layers)
    /// Pros: Smooth, clean transparency
    /// Cons: May have sorting issues with overlapping transparent objects
    AlphaBlend,
}

/// Occlusion mode for handling overhead voxels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OcclusionMode {
    /// No occlusion - voxels always visible
    None,

    /// Shader-based: per-pixel transparency based on camera-player ray
    /// Best for: Outdoor areas, small overhangs
    #[default]
    ShaderBased,

    /// Region-based: detect connected ceiling regions and hide entirely
    /// Best for: Buildings, rooms, caves
    RegionBased,

    /// Hybrid: Use region detection when inside, shader-based when outside
    /// Best for: Mixed environments
    Hybrid,
}

/// Shadow quality level controlling the directional light's shadow cascade configuration
/// and whether VoxelChunk meshes participate in shadow casting.
///
/// Lower levels reduce GPU shadow-pass overhead. `Low` is the default — it cuts the
/// profiled p95 frame spike from ~38ms to under 15ms while keeping nearby shadows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ShadowQuality {
    /// No shadows — `shadows_enabled: false`. Zero shadow-pass GPU cost.
    None,
    /// Only character/NPC meshes cast shadows. VoxelChunks get `NotShadowCaster`.
    /// Uses 2 cascades / 20-unit range to keep the shadow map alive for characters.
    CharactersOnly,
    /// Short-range voxel shadows: 2 cascades, 20-unit maximum distance.
    #[default]
    Low,
    /// Full-quality voxel shadows: 4 cascades, 100-unit maximum distance.
    /// Matches the original hard-coded shadow configuration.
    High,
}

/// Type alias for our extended occlusion material
pub type OcclusionMaterial = ExtendedMaterial<StandardMaterial, OcclusionExtension>;

/// Extension to StandardMaterial that adds occlusion transparency.
///
/// This extension provides the occlusion uniforms and overrides the fragment shader
/// to add dithered transparency based on player/camera positions.
#[derive(Asset, AsBindGroup, TypePath, Clone, Debug, Default)]
pub struct OcclusionExtension {
    /// Occlusion parameters passed to the shader
    #[uniform(100)]
    pub occlusion_uniforms: OcclusionUniforms,
}

impl MaterialExtension for OcclusionExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/occlusion_material.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/occlusion_material.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        // Minimal prepass: runs only dither + region discards, no PBR setup.
        // Avoids pbr_input_from_standard_material() which requires lighting
        // uniforms not bound during a depth-only prepass.
        "shaders/occlusion_material_prepass.wgsl".into()
    }
}

/// Uniform buffer for occlusion parameters.
///
/// This struct is uploaded to the GPU once per frame and used by all
/// fragments to calculate their transparency.
#[derive(Clone, Copy, Debug, ShaderType, PartialEq)]
pub struct OcclusionUniforms {
    /// Player world position (xyz)
    pub player_position: Vec3,
    /// Padding for alignment
    pub _padding1: f32,
    /// Camera world position (xyz)
    pub camera_position: Vec3,
    /// Padding for alignment
    pub _padding2: f32,
    /// Minimum alpha for fully occluded voxels (0.0 = invisible)
    pub min_alpha: f32,
    /// Horizontal radius for occlusion check in world units
    pub occlusion_radius: f32,
    /// Only affect voxels this much above player's Y position
    pub height_threshold: f32,
    /// Softness of the vertical falloff transition
    pub falloff_softness: f32,
    /// Transparency technique: 0 = Dithered, 1 = AlphaBlend
    pub technique: u32,
    /// Occlusion mode: 0 = None, 1 = ShaderBased, 2 = RegionBased, 3 = Hybrid
    pub mode: u32,
    /// Padding for 16-byte alignment
    pub _padding3: u32,
    pub _padding4: u32,
    /// Interior region minimum bounds (xyz), w = unused
    pub region_min: Vec4,
    /// Interior region maximum bounds (xyz), w = is_active (1.0 = active)
    pub region_max: Vec4,
}

impl Default for OcclusionUniforms {
    fn default() -> Self {
        Self {
            player_position: Vec3::ZERO,
            _padding1: 0.0,
            camera_position: Vec3::new(0.0, 10.0, 10.0),
            _padding2: 0.0,
            min_alpha: 0.15,
            occlusion_radius: 3.0,
            height_threshold: 0.5,
            falloff_softness: 2.0,
            technique: 0, // Dithered by default
            mode: 1,      // ShaderBased by default
            _padding3: 0,
            _padding4: 0,
            region_min: Vec4::ZERO,
            region_max: Vec4::ZERO, // w = 0 means inactive
        }
    }
}

/// Resource to store the shared occlusion material handle.
///
/// This handle is used by `update_occlusion_uniforms` to update the
/// material's uniform buffer each frame with current player/camera positions.
#[derive(Resource)]
pub struct OcclusionMaterialHandle(pub Handle<OcclusionMaterial>);

/// Configuration for the occlusion system.
///
/// Modify this resource at runtime to adjust the occlusion effect.
/// Changes are applied on the next frame.
#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct OcclusionConfig {
    /// Minimum alpha for fully occluded voxels (0.0 = invisible, 1.0 = opaque)
    pub min_alpha: f32,
    /// Horizontal occlusion radius in world units
    pub occlusion_radius: f32,
    /// Height threshold above player's Y position
    pub height_threshold: f32,
    /// Vertical falloff softness (larger = smoother transition)
    pub falloff_softness: f32,
    /// Whether occlusion is enabled
    pub enabled: bool,
    /// Transparency technique to use
    pub technique: TransparencyTechnique,
    /// Whether to show debug visualization (can toggle with F3)
    pub show_debug: bool,
    /// Occlusion mode (ShaderBased, RegionBased, or Hybrid)
    pub mode: OcclusionMode,
    /// Height threshold for interior detection (max ceiling height)
    pub interior_height_threshold: f32,
    /// Shadow quality level for the directional light.
    /// Controls cascade count, distance, and whether chunks cast shadows.
    pub shadow_quality: ShadowQuality,
    /// Update frequency for region detection (frames between updates)
    pub region_update_interval: u32,
}

impl Default for OcclusionConfig {
    fn default() -> Self {
        Self {
            min_alpha: 0.03, // Very transparent - barely noticeable
            occlusion_radius: 3.0,
            height_threshold: 0.5,
            falloff_softness: 2.0,
            enabled: true,
            // Dithered uses AlphaMode::Mask(0.001): the cutoff never fires (alpha always 1.0),
            // so it behaves like Opaque but sets MAY_DISCARD for the depth prepass.
            technique: TransparencyTechnique::Dithered,
            show_debug: false,
            mode: OcclusionMode::Hybrid, // Use hybrid mode for best results
            interior_height_threshold: 8.0, // Max ceiling height to trigger interior mode
            shadow_quality: ShadowQuality::Low,
            region_update_interval: 60, // Update every 60 frames (~1 time/sec at 60fps)
        }
    }
}

/// Private helper: config-driven uniform fields, used for Rust-side change tracking only.
/// Never sent to the GPU directly — assembled into [`OcclusionUniforms`] before writing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StaticOcclusionUniforms {
    min_alpha: f32,
    occlusion_radius: f32,
    height_threshold: f32,
    falloff_softness: f32,
    technique: u32,
    mode: u32,
}

impl StaticOcclusionUniforms {
    fn from_config(config: &OcclusionConfig) -> Self {
        let technique = match config.technique {
            TransparencyTechnique::Dithered => 0,
            TransparencyTechnique::AlphaBlend => 1,
        };
        let mode = match config.mode {
            OcclusionMode::None => 0,
            OcclusionMode::ShaderBased => 1,
            OcclusionMode::RegionBased => 2,
            OcclusionMode::Hybrid => 3,
        };
        Self {
            min_alpha: config.min_alpha,
            occlusion_radius: config.occlusion_radius,
            height_threshold: config.height_threshold,
            falloff_softness: config.falloff_softness,
            technique,
            mode,
        }
    }
}

/// Private helper: per-frame positional fields, used for Rust-side change tracking only.
/// Never sent to the GPU directly — assembled into [`OcclusionUniforms`] before writing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct DynamicOcclusionUniforms {
    player_position: Vec3,
    camera_position: Vec3,
    region_min: Vec4,
    region_max: Vec4,
}

impl DynamicOcclusionUniforms {
    fn new(
        player_position: Vec3,
        camera_position: Vec3,
        interior_state: Option<&InteriorState>,
    ) -> Self {
        let (region_min, region_max) = interior_state
            .and_then(|s| s.current_region.as_ref())
            .map(|r| {
                (
                    Vec4::new(r.min.x, r.min.y, r.min.z, 0.0),
                    Vec4::new(r.max.x, r.max.y, r.max.z, 1.0),
                )
            })
            .unwrap_or((Vec4::ZERO, Vec4::ZERO));
        Self {
            player_position,
            camera_position,
            region_min,
            region_max,
        }
    }
}

fn assemble_uniforms(s: &StaticOcclusionUniforms, d: &DynamicOcclusionUniforms) -> OcclusionUniforms {
    OcclusionUniforms {
        player_position: d.player_position,
        _padding1: 0.0,
        camera_position: d.camera_position,
        _padding2: 0.0,
        min_alpha: s.min_alpha,
        occlusion_radius: s.occlusion_radius,
        height_threshold: s.height_threshold,
        falloff_softness: s.falloff_softness,
        technique: s.technique,
        mode: s.mode,
        _padding3: 0,
        _padding4: 0,
        region_min: d.region_min,
        region_max: d.region_max,
    }
}

/// System to update occlusion material uniforms with fine-grained change detection.
///
/// Uses two private sub-struct caches — [`StaticOcclusionUniforms`] for config-driven
/// fields and [`DynamicOcclusionUniforms`] for per-frame positional fields — so that
/// each half is only recomputed when its inputs actually change. [`Assets::get_mut`] is
/// called (and the GPU re-upload triggered) only when at least one cache differs from
/// the newly computed value.
pub fn update_occlusion_uniforms(
    config: Res<OcclusionConfig>,
    camera_query: Query<Ref<Transform>, With<GameCamera>>,
    player_query: Query<Ref<Transform>, With<Player>>,
    material_handle: Option<Res<OcclusionMaterialHandle>>,
    mut materials: ResMut<Assets<OcclusionMaterial>>,
    interior_state: Option<Res<InteriorState>>,
    mut frame_counter: Local<u32>,
    mut static_cache: Local<Option<StaticOcclusionUniforms>>,
    mut dynamic_cache: Local<Option<DynamicOcclusionUniforms>>,
    profiler: Option<Res<FrameProfiler>>,
) {
    profile_scope!(profiler, "update_occlusion_uniforms");
    // Skip if disabled or mode is None
    if !config.enabled || config.mode == OcclusionMode::None {
        return;
    }

    // Skip if material handle not yet available
    let Some(material_handle) = material_handle else {
        *frame_counter += 1;
        if (*frame_counter).is_multiple_of(300) {
            info!("[Occlusion] Material handle not available yet (waiting for map to load)");
        }
        return;
    };

    let camera_ref = camera_query.single().ok();
    let player_ref = player_query.single().ok();

    // Recompute static fields only when OcclusionConfig changed or cache is empty.
    let new_static = if config.is_changed() || static_cache.is_none() {
        Some(StaticOcclusionUniforms::from_config(&config))
    } else {
        None
    };

    // Recompute dynamic fields only when positions/interior changed or cache is empty.
    let dynamic_input_changed = camera_ref.as_ref().map(|r| r.is_changed()).unwrap_or(false)
        || player_ref.as_ref().map(|r| r.is_changed()).unwrap_or(false)
        || interior_state.as_ref().map(|s| s.is_changed()).unwrap_or(false)
        || dynamic_cache.is_none();

    let new_dynamic = if dynamic_input_changed {
        let camera_pos = camera_ref
            .as_ref()
            .map(|t| t.translation)
            .unwrap_or(Vec3::new(0.0, 10.0, 10.0));
        let player_pos = player_ref
            .as_ref()
            .map(|t| t.translation)
            .unwrap_or(Vec3::ZERO);
        Some(DynamicOcclusionUniforms::new(
            player_pos,
            camera_pos,
            interior_state.as_deref(),
        ))
    } else {
        None
    };

    // Resolve effective values by copying out of Local (both types are Copy).
    // This ends any immutable borrow of the caches before the mutable update below.
    let effective_static: Option<StaticOcclusionUniforms> = new_static.or(*static_cache);
    let effective_dynamic: Option<DynamicOcclusionUniforms> = new_dynamic.or(*dynamic_cache);
    let (Some(s), Some(d)) = (effective_static, effective_dynamic) else {
        // Neither cache populated yet — only possible before first frame with a handle.
        return;
    };

    // Safety fallback: skip GPU work if neither sub-struct actually changed value.
    // This catches Bevy change-detection false positives on startup/state transitions.
    let static_dirty = new_static
        .as_ref()
        .map_or(false, |n| static_cache.as_ref() != Some(n));
    let dynamic_dirty = new_dynamic
        .as_ref()
        .map_or(false, |n| dynamic_cache.as_ref() != Some(n));

    if static_dirty || dynamic_dirty {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.extension.occlusion_uniforms = assemble_uniforms(&s, &d);
            if static_dirty {
                material.base.alpha_mode = match config.technique {
                    // Mask(0.001): sets MAY_DISCARD so the depth prepass runs our custom
                    // fragment shader. The threshold never fires (base_color.a is always 1.0),
                    // so visual behaviour is identical to Opaque but depth prepass is correct.
                    TransparencyTechnique::Dithered => AlphaMode::Mask(0.001),
                    TransparencyTechnique::AlphaBlend => AlphaMode::AlphaToCoverage,
                };
            }
            if let Some(new) = new_static {
                *static_cache = Some(new);
            }
            if let Some(new) = new_dynamic {
                *dynamic_cache = Some(new);
            }
        } else {
            *frame_counter += 1;
            if (*frame_counter).is_multiple_of(60) {
                warn!("[Occlusion] Material asset not found in Assets<OcclusionMaterial>");
            }
            return;
        }
    }

    *frame_counter += 1;
    if (*frame_counter).is_multiple_of(120) {
        let region_active = d.region_max.w > 0.5;
        info!(
            "[Occlusion] Mode: {:?}, Region active: {}, Player: ({:.1}, {:.1}, {:.1})",
            config.mode, region_active, d.player_position.x, d.player_position.y, d.player_position.z
        );
    }
}

/// Debug system to visualize occlusion zone.
///
/// Press F3 to toggle debug visualization showing:
/// - Yellow line: Camera to player ray
/// - Red circle: Occlusion radius above player
/// - Green line: Height threshold
/// - Cyan box: Interior region (when detected)
pub fn debug_draw_occlusion_zone(
    mut config: ResMut<OcclusionConfig>,
    mut gizmos: Gizmos,
    camera_query: Query<&Transform, With<GameCamera>>,
    player_query: Query<&Transform, With<Player>>,
    interior_state: Option<Res<InteriorState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut show_debug: Local<bool>,
) {
    // Toggle debug view with F3
    if keyboard.just_pressed(KeyCode::F3) {
        *show_debug = !*show_debug;
        config.show_debug = *show_debug;
    }

    if !*show_debug || !config.enabled {
        return;
    }

    let Ok(camera) = camera_query.single() else {
        return;
    };
    let Ok(player) = player_query.single() else {
        return;
    };

    // Draw ray from camera to player
    gizmos.line(
        camera.translation,
        player.translation,
        Color::srgba(1.0, 1.0, 0.0, 0.8),
    );

    // Draw occlusion cylinder above player (for shader-based mode)
    if matches!(config.mode, OcclusionMode::ShaderBased | OcclusionMode::Hybrid) {
        let cylinder_center = player.translation + Vec3::Y * (config.height_threshold + 2.0);
        gizmos.circle(
            Isometry3d::new(
                cylinder_center,
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ),
            config.occlusion_radius,
            Color::srgba(1.0, 0.0, 0.0, 0.5),
        );
    }

    // Draw height threshold line
    let threshold_y = player.translation.y + config.height_threshold;
    gizmos.line(
        Vec3::new(
            player.translation.x - 2.0,
            threshold_y,
            player.translation.z,
        ),
        Vec3::new(
            player.translation.x + 2.0,
            threshold_y,
            player.translation.z,
        ),
        Color::srgba(0.0, 1.0, 0.0, 0.8),
    );

    // Draw interior region bounds (for region-based or hybrid mode)
    if let Some(ref state) = interior_state {
        if let Some(ref region) = state.current_region {
            // Draw wireframe box for the interior region
            let color = Color::srgba(0.0, 1.0, 1.0, 0.8); // Cyan

            // Bottom face
            gizmos.line(Vec3::new(region.min.x, region.min.y, region.min.z), Vec3::new(region.max.x, region.min.y, region.min.z), color);
            gizmos.line(Vec3::new(region.max.x, region.min.y, region.min.z), Vec3::new(region.max.x, region.min.y, region.max.z), color);
            gizmos.line(Vec3::new(region.max.x, region.min.y, region.max.z), Vec3::new(region.min.x, region.min.y, region.max.z), color);
            gizmos.line(Vec3::new(region.min.x, region.min.y, region.max.z), Vec3::new(region.min.x, region.min.y, region.min.z), color);

            // Top face
            gizmos.line(Vec3::new(region.min.x, region.max.y, region.min.z), Vec3::new(region.max.x, region.max.y, region.min.z), color);
            gizmos.line(Vec3::new(region.max.x, region.max.y, region.min.z), Vec3::new(region.max.x, region.max.y, region.max.z), color);
            gizmos.line(Vec3::new(region.max.x, region.max.y, region.max.z), Vec3::new(region.min.x, region.max.y, region.max.z), color);
            gizmos.line(Vec3::new(region.min.x, region.max.y, region.max.z), Vec3::new(region.min.x, region.max.y, region.min.z), color);

            // Vertical edges
            gizmos.line(Vec3::new(region.min.x, region.min.y, region.min.z), Vec3::new(region.min.x, region.max.y, region.min.z), color);
            gizmos.line(Vec3::new(region.max.x, region.min.y, region.min.z), Vec3::new(region.max.x, region.max.y, region.min.z), color);
            gizmos.line(Vec3::new(region.max.x, region.min.y, region.max.z), Vec3::new(region.max.x, region.max.y, region.max.z), color);
            gizmos.line(Vec3::new(region.min.x, region.min.y, region.max.z), Vec3::new(region.min.x, region.max.y, region.max.z), color);
        }
    }
}

/// Plugin to set up the occlusion transparency system.
///
/// This plugin:
/// 1. Registers the `OcclusionMaterial` asset type
/// 2. Inserts the `OcclusionConfig` resource with defaults
/// 3. Adds the uniform update system to run every frame
/// 4. Adds interior detection system for region-based occlusion
///
/// Note: You must still:
/// - Create an `OcclusionMaterial` and store its handle in `OcclusionMaterialHandle`
/// - Use `MeshMaterial3d<OcclusionMaterial>` for your voxel chunks
pub struct OcclusionPlugin;

impl Plugin for OcclusionPlugin {
    fn build(&self, app: &mut App) {
        use bevy::prelude::in_state;
        use super::interior_detection::{detect_interior_system, InteriorState};
        use crate::states::GameState;

        app.add_plugins(MaterialPlugin::<OcclusionMaterial>::default())
            .insert_resource(OcclusionConfig::default())
            .insert_resource(InteriorState::default())
            .add_systems(
                Update,
                (
                    detect_interior_system,
                    update_occlusion_uniforms,
                    debug_draw_occlusion_zone,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame).or(in_state(GameState::Paused))),
            );
    }
}

/// Helper function to create an occlusion material with specified technique.
///
/// This creates an ExtendedMaterial combining StandardMaterial (for PBR/shadows)
/// with OcclusionExtension (for transparency).
///
/// Use this when spawning voxel chunks:
/// ```rust,ignore
/// let material_handle = create_occlusion_material(&mut materials, TransparencyTechnique::AlphaBlend);
/// commands.insert_resource(OcclusionMaterialHandle(material_handle.clone()));
///
/// // Use material_handle for all chunks
/// commands.spawn((
///     Mesh3d(mesh),
///     MeshMaterial3d(material_handle.clone()),
///     // ...
/// ));
/// ```
pub fn create_occlusion_material(
    materials: &mut Assets<OcclusionMaterial>,
    technique: TransparencyTechnique,
) -> Handle<OcclusionMaterial> {
    let alpha_mode = match technique {
        // Mask(0.001): sets MAY_DISCARD so the depth prepass runs our custom fragment shader.
        // The cutoff never fires (base_color.a is always 1.0), visual result identical to Opaque.
        TransparencyTechnique::Dithered => AlphaMode::Mask(0.001),
        TransparencyTechnique::AlphaBlend => AlphaMode::AlphaToCoverage, // Uses MSAA for smooth transparency without sorting issues
    };

    materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.9,
            metallic: 0.0,
            reflectance: 0.1,
            alpha_mode,
            ..default()
        },
        extension: OcclusionExtension::default(),
    })
}

#[cfg(test)]
mod tests;
