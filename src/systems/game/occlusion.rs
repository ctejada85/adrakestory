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
//! - `technique`: Dithered (screen-door) or AlphaBlend (smooth like Photoshop)
//! - `mode`: ShaderBased, RegionBased, or Hybrid occlusion mode

use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

use super::components::{GameCamera, Player};
use super::interior_detection::InteriorState;

/// Transparency technique for occlusion effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
}

/// Uniform buffer for occlusion parameters.
///
/// This struct is uploaded to the GPU once per frame and used by all
/// fragments to calculate their transparency.
#[derive(Clone, Copy, Debug, ShaderType)]
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
#[derive(Resource)]
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
    /// Whether to hide shadows of occluded voxels (future use)
    pub hide_shadows: bool,
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
            technique: TransparencyTechnique::AlphaBlend, // Smooth transparency by default
            show_debug: false,
            mode: OcclusionMode::Hybrid, // Use hybrid mode for best results
            interior_height_threshold: 8.0, // Max ceiling height to trigger interior mode
            hide_shadows: true,
            region_update_interval: 10, // Update every 10 frames (~6 times/sec at 60fps)
        }
    }
}

/// System to update occlusion material uniforms each frame.
///
/// This system runs in O(1) time - it just copies a few floats to the
/// uniform buffer. The actual occlusion calculation happens on the GPU.
pub fn update_occlusion_uniforms(
    config: Res<OcclusionConfig>,
    camera_query: Query<&Transform, With<GameCamera>>,
    player_query: Query<&Transform, With<Player>>,
    material_handle: Option<Res<OcclusionMaterialHandle>>,
    mut materials: ResMut<Assets<OcclusionMaterial>>,
    interior_state: Option<Res<InteriorState>>,
    mut frame_counter: Local<u32>,
) {
    // Skip if disabled or mode is None
    if !config.enabled || config.mode == OcclusionMode::None {
        return;
    }

    // Skip if material handle not yet available
    let Some(material_handle) = material_handle else {
        // Debug: Log when material handle is missing
        *frame_counter += 1;
        // Only log every 300 frames (5 seconds at 60fps) since this is expected during loading
        if (*frame_counter).is_multiple_of(300) {
            info!("[Occlusion] Material handle not available yet (waiting for map to load)");
        }
        return;
    };

    // Get camera position (fall back to default if not found)
    let camera_pos = camera_query
        .get_single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::new(0.0, 10.0, 10.0));

    // Get player position (fall back to origin if not found)
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    // Get interior region bounds if available
    let (region_min, region_max) = interior_state
        .as_ref()
        .and_then(|state| state.current_region.as_ref())
        .map(|region| {
            (
                Vec4::new(region.min.x, region.min.y, region.min.z, 0.0),
                Vec4::new(region.max.x, region.max.y, region.max.z, 1.0), // w = 1.0 means active
            )
        })
        .unwrap_or((Vec4::ZERO, Vec4::ZERO)); // w = 0.0 means inactive

    // Update the material's uniform buffer (accessing the extension part)
    if let Some(material) = materials.get_mut(&material_handle.0) {
        let technique_value = match config.technique {
            TransparencyTechnique::Dithered => 0,
            TransparencyTechnique::AlphaBlend => 1,
        };

        let mode_value = match config.mode {
            OcclusionMode::None => 0,
            OcclusionMode::ShaderBased => 1,
            OcclusionMode::RegionBased => 2,
            OcclusionMode::Hybrid => 3,
        };

        material.extension.occlusion_uniforms = OcclusionUniforms {
            player_position: player_pos,
            _padding1: 0.0,
            camera_position: camera_pos,
            _padding2: 0.0,
            min_alpha: config.min_alpha,
            occlusion_radius: config.occlusion_radius,
            height_threshold: config.height_threshold,
            falloff_softness: config.falloff_softness,
            technique: technique_value,
            mode: mode_value,
            _padding3: 0,
            _padding4: 0,
            region_min,
            region_max,
        };

        // Debug: Log uniforms every 120 frames (about 2 seconds at 60fps)
        *frame_counter += 1;
        if (*frame_counter).is_multiple_of(120) {
            let region_active = region_max.w > 0.5;
            info!(
                "[Occlusion] Mode: {:?}, Region active: {}, Player: ({:.1}, {:.1}, {:.1})",
                config.mode, region_active, player_pos.x, player_pos.y, player_pos.z
            );
        }
    } else {
        *frame_counter += 1;
        if (*frame_counter).is_multiple_of(60) {
            warn!("[Occlusion] Material asset not found in Assets<OcclusionMaterial>");
        }
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

    let Ok(camera) = camera_query.get_single() else {
        return;
    };
    let Ok(player) = player_query.get_single() else {
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
        use super::interior_detection::{detect_interior_system, InteriorState};

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
                    .chain(),
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
        TransparencyTechnique::Dithered => AlphaMode::Opaque, // Dithered uses discard, not blending
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
