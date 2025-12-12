//! Occlusion transparency shader for voxel chunks.
//!
//! This shader calculates per-pixel transparency based on the fragment's
//! position relative to the player and camera, making voxels above the
//! player transparent so the player remains visible.
//!
//! Uses Bevy's forward_io for proper vertex attribute handling.

#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
}

// Custom uniforms for occlusion
struct OcclusionUniforms {
    player_position: vec3<f32>,
    _padding1: f32,
    camera_position: vec3<f32>,
    _padding2: f32,
    min_alpha: f32,
    occlusion_radius: f32,
    height_threshold: f32,
    falloff_softness: f32,
}

@group(2) @binding(0)
var<uniform> base_color: vec4<f32>;

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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get base color from vertex colors (set by Bevy when mesh has ATTRIBUTE_COLOR)
#ifdef VERTEX_COLORS
    var color = in.color;
#else
    // Fallback to uniform base_color if no vertex colors
    var color = base_color;
#endif
    
    // Calculate occlusion alpha using world position
    let occlusion_alpha = calculate_occlusion_alpha(in.world_position.xyz);
    
    // Apply occlusion transparency
    let final_alpha = color.a * occlusion_alpha;
    
    // Discard fully transparent fragments
    if final_alpha < 0.01 {
        discard;
    }
    
    // Use dithered transparency to avoid alpha sorting issues
    // This creates a screen-door effect but eliminates flickering
    if !dither_check(in.position.xy, final_alpha) {
        discard;
    }
    
    // Output the vertex color directly (no custom lighting - use scene lights)
    return vec4<f32>(color.rgb, 1.0);
}
