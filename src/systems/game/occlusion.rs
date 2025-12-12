//! Voxel occlusion transparency system using custom shaders.
//!
//! This module provides a custom material that makes voxels transparent
//! when they block the camera's view of the player character. The transparency
//! is calculated per-pixel in the fragment shader for smooth, high-quality results.
//!
//! ## Usage
//!
//! 1. Add `OcclusionPlugin` to your app
//! 2. Use `OcclusionMaterial` instead of `StandardMaterial` for voxel chunks
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

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

use super::components::{GameCamera, Player};

/// Custom material for voxel occlusion transparency.
///
/// This material calculates per-pixel transparency based on the fragment's
/// position relative to the player and camera. All chunks can share a single
/// instance of this material, maintaining GPU instancing efficiency.
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct OcclusionMaterial {
    /// Base color multiplier (typically white, actual colors come from vertex colors)
    #[uniform(0)]
    pub base_color: LinearRgba,

    /// Occlusion parameters passed to the shader
    #[uniform(100)]
    pub occlusion_uniforms: OcclusionUniforms,
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
        }
    }
}

impl Default for OcclusionMaterial {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::WHITE,
            occlusion_uniforms: OcclusionUniforms::default(),
        }
    }
}

impl Material for OcclusionMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/occlusion_material.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/occlusion_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
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
}

impl Default for OcclusionConfig {
    fn default() -> Self {
        Self {
            min_alpha: 0.15,
            occlusion_radius: 3.0,
            height_threshold: 0.5,
            falloff_softness: 2.0,
            enabled: true,
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
) {
    // Skip if disabled
    if !config.enabled {
        return;
    }

    // Skip if material handle not yet available
    let Some(material_handle) = material_handle else {
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

    // Update the material's uniform buffer
    if let Some(material) = materials.get_mut(&material_handle.0) {
        material.occlusion_uniforms = OcclusionUniforms {
            player_position: player_pos,
            _padding1: 0.0,
            camera_position: camera_pos,
            _padding2: 0.0,
            min_alpha: config.min_alpha,
            occlusion_radius: config.occlusion_radius,
            height_threshold: config.height_threshold,
            falloff_softness: config.falloff_softness,
        };
    }
}

/// Debug system to visualize occlusion zone.
///
/// Press F3 to toggle debug visualization showing:
/// - Yellow line: Camera to player ray
/// - Red circle: Occlusion radius above player
/// - Green line: Height threshold
#[allow(dead_code)]
pub fn debug_draw_occlusion_zone(
    config: Res<OcclusionConfig>,
    mut gizmos: Gizmos,
    camera_query: Query<&Transform, With<GameCamera>>,
    player_query: Query<&Transform, With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut show_debug: Local<bool>,
) {
    // Toggle debug view with F3
    if keyboard.just_pressed(KeyCode::F3) {
        *show_debug = !*show_debug;
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

    // Draw occlusion cylinder above player
    let cylinder_center = player.translation + Vec3::Y * (config.height_threshold + 2.0);
    gizmos.circle(
        Isometry3d::new(
            cylinder_center,
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        ),
        config.occlusion_radius,
        Color::srgba(1.0, 0.0, 0.0, 0.5),
    );

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
}

/// Plugin to set up the occlusion transparency system.
///
/// This plugin:
/// 1. Registers the `OcclusionMaterial` asset type
/// 2. Inserts the `OcclusionConfig` resource with defaults
/// 3. Adds the uniform update system to run every frame
///
/// Note: You must still:
/// - Create an `OcclusionMaterial` and store its handle in `OcclusionMaterialHandle`
/// - Use `MeshMaterial3d<OcclusionMaterial>` for your voxel chunks
pub struct OcclusionPlugin;

impl Plugin for OcclusionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<OcclusionMaterial>::default())
            .insert_resource(OcclusionConfig::default())
            .add_systems(Update, update_occlusion_uniforms);

        // Optionally add debug visualization (uncomment to enable)
        // app.add_systems(Update, debug_draw_occlusion_zone);
    }
}

/// Helper function to create an occlusion material with default settings.
///
/// Use this when spawning voxel chunks:
/// ```rust,ignore
/// let material_handle = create_occlusion_material(&mut materials);
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
) -> Handle<OcclusionMaterial> {
    materials.add(OcclusionMaterial::default())
}
