//! Minimal depth prepass shader for OcclusionMaterial.
//!
//! Runs only the discard logic (dither + region checks) that determines which
//! fragments write depth. No PBR setup, no lighting, no texture sampling.
//!
//! The depth prepass populates the depth buffer before the main forward pass so
//! occluded fragments fail the hardware depth test without running the expensive
//! PBR + occlusion shader.
//!
//! base_color.a is always 1.0 for voxel chunks (AlphaMode::Opaque), so
//! final_alpha == occlusion_alpha and we can skip pbr_input entirely.

// For depth-only prepass (DepthPrepass without NormalPrepass), PREPASS_FRAGMENT is NOT defined
// and FragmentOutput does not exist. The fragment function uses no return type — depth is written
// automatically from the fragment position. Only discard logic is needed here.
#import bevy_pbr::prepass_io::VertexOutput

// --- Occlusion uniform (must match binding in occlusion_material.wgsl) -------

struct OcclusionUniforms {
    player_position: vec3<f32>,
    _padding1: f32,
    camera_position: vec3<f32>,
    _padding2: f32,
    min_alpha: f32,
    occlusion_radius: f32,
    height_threshold: f32,
    falloff_softness: f32,
    technique: u32,
    mode: u32,
    _padding3: u32,
    _padding4: u32,
    region_min: vec4<f32>,
    region_max: vec4<f32>,
}

@group(2) @binding(100)
var<uniform> occlusion: OcclusionUniforms;

// --- Helpers (copied from occlusion_material.wgsl) ---------------------------

const BAYER_MATRIX: array<f32, 16> = array<f32, 16>(
     0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
    12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0,
     3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
    15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0
);

const XZ_MARGIN_FACTOR: f32 = 2.0;

fn point_to_ray_distance_xz(point: vec3<f32>, ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> f32 {
    let point_xz = vec2<f32>(point.x, point.z);
    let origin_xz = vec2<f32>(ray_origin.x, ray_origin.z);
    let dir_xz_raw = vec2<f32>(ray_dir.x, ray_dir.z);
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

fn calculate_occlusion_alpha(world_pos: vec3<f32>) -> f32 {
    let player_y = occlusion.player_position.y;
    let fragment_y = world_pos.y;
    if fragment_y <= player_y + occlusion.height_threshold {
        return 1.0;
    }
    let xz_offset = world_pos.xz - occlusion.player_position.xz;
    let xz_dist_sq = dot(xz_offset, xz_offset);
    let xz_margin = occlusion.occlusion_radius * XZ_MARGIN_FACTOR;
    if xz_dist_sq > xz_margin * xz_margin {
        return 1.0;
    }
    let ray_direction = normalize(occlusion.player_position - occlusion.camera_position);
    let horizontal_distance = point_to_ray_distance_xz(
        world_pos,
        occlusion.camera_position,
        ray_direction
    );
    if horizontal_distance >= occlusion.occlusion_radius {
        return 1.0;
    }
    let edge_softness = 0.5;
    let soft_distance = smoothstep(
        occlusion.occlusion_radius - edge_softness,
        occlusion.occlusion_radius,
        horizontal_distance
    );
    let height_factor = smoothstep(
        player_y + occlusion.height_threshold,
        player_y + occlusion.height_threshold + occlusion.falloff_softness,
        fragment_y
    );
    let base_alpha = mix(occlusion.min_alpha, 1.0, soft_distance);
    return mix(1.0, base_alpha, height_factor);
}

fn dither_check(screen_pos: vec2<f32>, alpha: f32) -> bool {
    let x = u32(screen_pos.x) % 4u;
    let y = u32(screen_pos.y) % 4u;
    let threshold = BAYER_MATRIX[y * 4u + x];
    return alpha > threshold;
}

fn in_interior_region(world_pos: vec3<f32>) -> bool {
    if occlusion.region_max.w < 0.5 {
        return false;
    }
    let epsilon = 0.01;
    return world_pos.x > occlusion.region_min.x + epsilon &&
           world_pos.x < occlusion.region_max.x - epsilon &&
           world_pos.y > occlusion.region_min.y + epsilon &&
           world_pos.z > occlusion.region_min.z + epsilon &&
           world_pos.z < occlusion.region_max.z - epsilon;
}

// --- Entry point -------------------------------------------------------------

// No return type: depth is written implicitly from fragment position.
// This is valid for depth-only prepass (no color attachments, no PREPASS_FRAGMENT define).
@fragment
fn fragment(in: VertexOutput) {
    let world_pos = in.world_position.xyz;

    // Region-based discard — hide interior regions entirely.
    if occlusion.mode == 2u || occlusion.mode == 3u {
        if in_interior_region(world_pos) {
            discard;
        }
    }

    // Shader-based dither discard.
    // base_color.a is always 1.0 for voxel chunks, so final_alpha == occlusion_alpha.
    // AlphaBlend mode (technique == 1) does not use discard — MSAA handles it.
    if occlusion.mode == 1u && occlusion.technique == 0u {
        let occlusion_alpha = calculate_occlusion_alpha(world_pos);
        if occlusion_alpha < 0.01 {
            discard;
        }
        if !dither_check(in.position.xy, occlusion_alpha) {
            discard;
        }
    }

    // Depth written automatically from fragment position — no return needed.
}
