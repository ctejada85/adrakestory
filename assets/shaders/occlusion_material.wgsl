//! Occlusion transparency shader for voxel chunks.
//!
//! This shader calculates per-pixel transparency based on the fragment's
//! position relative to the player and camera, making voxels above the
//! player transparent so the player remains visible.

#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
    mesh_view_bindings::view,
}

// Vertex input structure
struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
#ifdef VERTEX_COLORS
    @location(3) color: vec4<f32>,
#endif
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

// Vertex output structure
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
#ifdef VERTEX_COLORS
    @location(3) color: vec4<f32>,
#endif
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    let world_position = mesh_functions::mesh_position_local_to_world(
        mesh_functions::get_world_from_local(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0)
    );
    
    out.clip_position = position_world_to_clip(world_position.xyz);
    out.world_position = world_position.xyz;
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index
    );
    out.uv = vertex.uv;
    
#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif
    
    return out;
}

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
    
    // Smooth falloff based on distance from ray center
    let distance_factor = horizontal_distance / occlusion.occlusion_radius;
    
    // Smooth falloff based on height above threshold
    let height_factor = smoothstep(
        player_y + occlusion.height_threshold,
        player_y + occlusion.height_threshold + occlusion.falloff_softness,
        fragment_y
    );
    
    // Combine factors: closer to ray and higher above player = more transparent
    let base_alpha = mix(occlusion.min_alpha, 1.0, distance_factor);
    return mix(1.0, base_alpha, height_factor);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get base color from vertex colors or uniform
#ifdef VERTEX_COLORS
    var color = in.color;
#else
    var color = base_color;
#endif
    
    // Calculate occlusion alpha
    let occlusion_alpha = calculate_occlusion_alpha(in.world_position);
    
    // Apply basic lighting (simplified - ambient + directional)
    let light_dir = normalize(vec3<f32>(0.4, 0.8, 0.3));
    let ndotl = max(dot(normalize(in.world_normal), light_dir), 0.0);
    let ambient = 0.35;
    let diffuse = ndotl * 0.65;
    
    let lit_color = color.rgb * (ambient + diffuse);
    
    // Apply occlusion transparency
    let final_alpha = color.a * occlusion_alpha;
    
    // Discard fully transparent fragments for performance
    if final_alpha < 0.01 {
        discard;
    }
    
    return vec4<f32>(lit_color, final_alpha);
}
