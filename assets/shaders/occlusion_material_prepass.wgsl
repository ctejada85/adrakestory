//! Minimal depth prepass shader for OcclusionMaterial.
//!
//! Determines which fragments write depth. No PBR setup, no lighting, no texture sampling.
//!
//! The depth prepass populates the depth buffer before the main forward pass so
//! objects behind transparent voxels (e.g. the player) are not blocked by the
//! wrong depth value.
//!
//! Discard rule: voxels ABOVE the character level are discarded (don't write depth);
//! voxels BELOW are kept. The main pass handles the visual dither/region pattern.

// For depth-only prepass (DepthPrepass without NormalPrepass), PREPASS_FRAGMENT is NOT defined
// and FragmentOutput does not exist. The fragment function uses no return type — depth is written
// automatically from the fragment position. Only discard logic is needed here.
#import bevy_pbr::prepass_io::VertexOutput
#import bevy_render::view::View

// --- View uniform (group 0, binding 0) ----------------------------------------
// Used to detect shadow passes: projection[3][3] == 1.0 → orthographic → shadow map pass.

@group(0) @binding(0)
var<uniform> view: View;

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

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> occlusion: OcclusionUniforms;

// --- Helpers ------------------------------------------------------------------

const XZ_MARGIN_FACTOR: f32 = 2.0;

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

    // projection[3][3]:
    //   1.0 = orthographic  → directional light shadow map pass  → skip discards
    //   0.0 = perspective   → player camera depth prepass        → apply discards
    //
    // Directional light shadow maps use orthographic projection; the player camera
    // uses perspective. This check is Bevy-version-agnostic (pure WGSL math).
    //
    // NOTE: Point/spot light shadow passes also use perspective, so discards fire
    // for those too. Acceptable — both have shadows_enabled: false in current maps.
    let is_shadow_pass = view.projection[3][3] >= 0.5;

    if !is_shadow_pass {
        // Region-based discard — hide interior regions entirely.
        if occlusion.mode == 2u || occlusion.mode == 3u {
            if in_interior_region(world_pos) {
                discard;
            }
        }

        // Shader-based height discard (mode 1 or 3, dithered technique).
        //
        // The prepass must make a BINARY keep/discard decision per fragment.
        // Using dither here would leave some above-player fragments in the depth buffer,
        // which would block the player (lower reverse-Z depth fails GreaterEqual test).
        //
        // Rule: discard fragments ABOVE the player level; keep fragments BELOW.
        // The XZ proximity guard avoids discarding distant above-height geometry that
        // the main pass would keep opaque (over-discarding is safe — the main pass
        // re-writes depth for any fragment it renders that the prepass skipped).
        if (occlusion.mode == 1u || occlusion.mode == 3u) && occlusion.technique == 0u {
            if world_pos.y > occlusion.player_position.y + occlusion.height_threshold {
                let xz_offset = world_pos.xz - occlusion.player_position.xz;
                let xz_dist_sq = dot(xz_offset, xz_offset);
                let xz_margin = occlusion.occlusion_radius * XZ_MARGIN_FACTOR;
                if xz_dist_sq <= xz_margin * xz_margin {
                    discard;
                }
            }
        }
    }

    // Depth written automatically from fragment position — no return needed.
}
