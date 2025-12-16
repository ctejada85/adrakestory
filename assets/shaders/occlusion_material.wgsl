//! Occlusion transparency shader for voxel chunks.
//!
//! This shader extends StandardMaterial to add per-pixel occlusion transparency
//! based on the fragment's position relative to the player and camera.
//! Voxels above the player become transparent so the player remains visible.
//!
//! Supports multiple occlusion modes:
//! - ShaderBased: Per-pixel transparency based on camera-player ray
//! - RegionBased: Hide voxels within detected interior region bounds
//! - Hybrid: Use region detection when inside, shader-based when outside
//!
//! Uses pbr_input_from_standard_material for proper PBR lighting with shadows.

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

// Custom uniforms for occlusion (binding 100 to avoid conflict with StandardMaterial)
struct OcclusionUniforms {
    player_position: vec3<f32>,
    _padding1: f32,
    camera_position: vec3<f32>,
    _padding2: f32,
    min_alpha: f32,
    occlusion_radius: f32,
    height_threshold: f32,
    falloff_softness: f32,
    // Transparency technique: 0 = Dithered, 1 = AlphaBlend
    technique: u32,
    // Occlusion mode: 0 = None, 1 = ShaderBased, 2 = RegionBased, 3 = Hybrid
    mode: u32,
    _padding3: u32,
    _padding4: u32,
    // Interior region bounds (xyz = bounds, region_max.w = is_active)
    region_min: vec4<f32>,
    region_max: vec4<f32>,
}

@group(2) @binding(100)
var<uniform> occlusion: OcclusionUniforms;

// Bayer matrix for 4x4 ordered dithering
// This creates a screen-door transparency effect without alpha blending
const BAYER_MATRIX: array<f32, 16> = array<f32, 16>(
     0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
    12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0,
     3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
    15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0
);

// Calculate distance from point to ray in XZ plane (ignoring Y)
fn point_to_ray_distance_xz(point: vec3<f32>, ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> f32 {
    let point_xz = vec2<f32>(point.x, point.z);
    let origin_xz = vec2<f32>(ray_origin.x, ray_origin.z);
    let dir_xz_raw = vec2<f32>(ray_dir.x, ray_dir.z);
    
    // Handle near-vertical rays
    let dir_length = length(dir_xz_raw);
    if dir_length < 0.001 {
        return length(point_xz - origin_xz);
    }
    
    let dir_xz = dir_xz_raw / dir_length;
    let to_point = point_xz - origin_xz;
    let projection = dot(to_point, dir_xz);
    let closest_point = origin_xz + dir_xz * projection;
    return length(point_xz - closest_point);
}

// Calculate occlusion alpha for a world position
fn calculate_occlusion_alpha(world_pos: vec3<f32>) -> f32 {
    let player_y = occlusion.player_position.y;
    let fragment_y = world_pos.y;
    
    // Only apply occlusion to fragments above the player
    if fragment_y <= player_y + occlusion.height_threshold {
        return 1.0;
    }
    
    // Calculate ray from camera to player
    let ray_direction = normalize(occlusion.player_position - occlusion.camera_position);
    
    // Distance from fragment to camera-player ray (XZ plane)
    let horizontal_distance = point_to_ray_distance_xz(
        world_pos,
        occlusion.camera_position,
        ray_direction
    );
    
    // Check if within occlusion radius
    if horizontal_distance >= occlusion.occlusion_radius {
        return 1.0;
    }
    
    // Smooth falloff based on distance from ray center (soft edges)
    let edge_softness = 0.5;
    let soft_distance = smoothstep(
        occlusion.occlusion_radius - edge_softness,
        occlusion.occlusion_radius,
        horizontal_distance
    );
    
    // Smooth falloff based on height above threshold
    let height_factor = smoothstep(
        player_y + occlusion.height_threshold,
        player_y + occlusion.height_threshold + occlusion.falloff_softness,
        fragment_y
    );
    
    // Combine factors: closer to ray and higher above player = more transparent
    let base_alpha = mix(occlusion.min_alpha, 1.0, soft_distance);
    return mix(1.0, base_alpha, height_factor);
}

// Check if fragment should be discarded based on dithering pattern
// Returns true if the fragment should be visible
fn dither_check(screen_pos: vec2<f32>, alpha: f32) -> bool {
    let x = u32(screen_pos.x) % 4u;
    let y = u32(screen_pos.y) % 4u;
    let index = y * 4u + x;
    let threshold = BAYER_MATRIX[index];
    return alpha > threshold;
}

// Check if fragment is inside the interior region bounds
// Uses a small inset to avoid z-fighting at boundaries
fn in_interior_region(world_pos: vec3<f32>) -> bool {
    // Check if region is active (region_max.w > 0.5)
    if occlusion.region_max.w < 0.5 {
        return false;
    }
    
    // Add small epsilon inset to avoid artifacts at exact boundaries
    let epsilon = 0.01;
    
    return world_pos.x > occlusion.region_min.x + epsilon &&
           world_pos.x < occlusion.region_max.x - epsilon &&
           world_pos.y > occlusion.region_min.y + epsilon &&
           world_pos.z > occlusion.region_min.z + epsilon &&
           world_pos.z < occlusion.region_max.z - epsilon;
    // Note: No upper Y check - we want to hide everything above ceiling_y
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let world_pos = in.world_position.xyz;
    
    // Early discard for region-based occlusion (must happen before any other processing)
    // This ensures both prepass and main pass discard the same fragments
    if occlusion.mode == 2u || occlusion.mode == 3u {
        if in_interior_region(world_pos) {
            discard;
        }
    }
    
    // Generate PbrInput from StandardMaterial bindings (includes all shadow data)
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    
    // Apply shader-based occlusion ONLY for mode 1 (ShaderBased)
    // Mode 3 (Hybrid) fallback is temporarily disabled
    if occlusion.mode == 1u {
        let occlusion_alpha = calculate_occlusion_alpha(world_pos);
        let final_alpha = pbr_input.material.base_color.a * occlusion_alpha;
        
        if final_alpha < 0.01 {
            discard;
        }
        
        if occlusion.technique == 0u {
            // Dithered transparency
            if !dither_check(in.position.xy, final_alpha) {
                discard;
            }
        } else {
            // Alpha blend transparency
            pbr_input.material.base_color.a = final_alpha;
        }
    }
    
    // Standard alpha discard from material settings
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // In deferred mode, output deferred data
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // Apply PBR lighting (includes shadows, ambient, etc.)
    out.color = apply_pbr_lighting(pbr_input);
    // Post-processing (fog, tonemapping, etc.)
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    
    // For alpha blend mode (AlphaToCoverage), set alpha for MSAA hardware to handle
    if occlusion.technique == 1u && occlusion.mode != 0u {
        out.color.a = pbr_input.material.base_color.a;
    }
#endif

    return out;
}
