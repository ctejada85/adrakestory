# Occlusion Rendering Pipeline

Shows how each frame renders voxel chunks with occlusion transparency so the
player remains visible when walking inside buildings.

## Frame Pipeline

```mermaid
flowchart TD
    A([Frame Start]) --> B

    subgraph INTERIOR ["detect_interior_system (CPU)"]
        D{"Frame throttle: frames_since_update\nlt region_update_interval?"}
        D -- Skip --> SKIP([return, keep last region])
        D -- Run --> D1{"Player moved less than\n0.3 units since last detection?"}
        D1 -- "Yes, skip" --> SKIP
        D1 -- No --> D2{"SubVoxel entities added\nor removed this frame?"}
        D2 -- "Yes, spawn in progress" --> D3["Set rebuild_pending = true, return early"]
        D2 -- "No, settled" --> D4{"rebuild_pending == true?"}
        D4 -- Yes --> D5["Clear occupied_voxels_cache\nrebuild_pending = false"]
        D4 -- No --> D6
        D5 --> D6
        D6{"occupied_voxels_cache present?"}
        D6 -- No --> D7["build_occupied_voxel_set\nIterate SpatialGrid cells\nSub-voxel bounds.min to floor to IVec3\nResult: HashSet of occupied voxel positions"]
        D7 --> D8
        D6 -- "Yes, reuse cache" --> D8
        D8["Check 4 XZ neighbour voxels the player overlaps\nfloor and floor+1 on both X and Z axes"]
        D8 --> D9
        D9["find_ceiling_voxel_above  (x4 positions)\nScan Y = player_y+2 to player_y+height_threshold\nFor each solid voxel found:\n  gap check: no solid between player and it?\n  Yes = ceiling found, return Y\n  No  = wall, keep scanning\nReturn None if nothing found"]
        D9 --> D10{"Any ceiling found\nin any of the 4 spots?"}
        D10 -- "Yes, ceiling found" --> D11
        D10 -- No --> D12{"Player still inside\nexisting region AABB (XZ only)?"}
        D12 -- Yes --> D13["Keep current_region\nhysteresis: player near wall"]
        D12 -- No --> D14["current_region = None\nPlayer is outside"]
        D11["flood_fill_ceiling_region_voxel\nBFS queue, 4-connected XZ plane\nY search band: ceiling_y-1 to ceiling_y+2\nExpand to neighbour if current OR neighbour\nhas a ceiling voxel (crosses small gaps)\nCap: MAX_REGION_SIZE = 1000 voxels\nMax search radius: 30 Manhattan steps\nTrack min_x, max_x, min_z, max_z"]
        D11 --> D15{"voxel_count >= 2?"}
        D15 -- Yes --> D16["Build InteriorRegion AABB\nmin.y = (player_y+2) - 0.5  hide from 2 voxels above player\nmax.y = ceiling_y + 100     hide everything above ceiling\nXZ = flood-fill extents +/- 0.5 voxel padding"]
        D15 -- "No, noise" --> D13
        D16 --> D17["current_region = Some(InteriorRegion)"]
    end

    subgraph CPU ["CPU — Occlusion Update System"]
        B["Player and camera positions collected from ECS"]
        B --> C{"Interior detection mode active?\nmode = RegionBased or Hybrid"}
        C -- Yes --> D
        C -- No --> E2["region_max.w = 0, region inactive"]
        D17 --> E["InteriorState::current_region\nOption of AABB passed to shader"]
        E --> F
        E2 --> F
        F["Assemble OcclusionUniforms:\nplayer_pos, camera_pos,\nmode, technique, region bounds"]
        F --> G["Upload uniform buffer to GPU\nvia ExtendedMaterial"]
    end

    A --> B
    G --> H

    subgraph PREPASS ["GPU — Depth Prepass (DepthPrepass on camera)"]
        H["Vertex shader: transform voxel chunk vertices\nwrite world_position"]
        H --> I{"technique == Dithered\nAND mode != None?"}
        I -- Yes --> J{"world_pos.y > player_y + height_threshold\nAND within XZ radius?"}
        J -- Yes --> K([discard, do not write depth])
        J -- No --> L([write depth, fragment kept])
        I -- No --> L
        H --> M{"mode == RegionBased or Hybrid\nAND region active?"}
        M -- "in region" --> K
        M -- "outside or inactive" --> L
    end

    L --> N

    subgraph MAINPASS ["GPU — Main Forward Pass (AlphaMask3d render phase)"]
        N["Early-Z test vs prepass depth\nFragments above player fail GreaterEqual\nand are skipped entirely"]
        N --> O{"Survive depth test?"}
        O -- "No, culled" --> P([Fragment discarded by hardware])
        O -- Yes --> Q
        Q["pbr_input_from_standard_material\nSample shadow maps, lighting"]
        Q --> R{"mode == RegionBased or Hybrid\nAND in_interior_region?"}
        R -- Yes --> S([discard])
        R -- No --> T
        T{"mode == ShaderBased or Hybrid?"}
        T -- Yes --> U["calculate_occlusion_alpha\nHeight + XZ ray distance\nsmooth falloff"]
        U --> V{"technique?"}
        V -- Dithered --> W{"dither_check\nBayer 4x4 matrix"}
        W -- "alpha <= threshold" --> S
        W -- "alpha > threshold" --> X
        V -- AlphaBlend --> Y["Set base_color.a\nAlphaToCoverage blending"]
        Y --> X
        T -- "No / None" --> X
        X["apply_pbr_lighting\nDirectional + point lights, shadows"]
        X --> Z["main_pass_post_lighting_processing\nFog, tonemapping"]
        Z --> AA([Write to color buffer])
    end

    AA --> AB([Frame output])
```

## Occlusion Modes

| Mode | CPU side | Prepass | Main pass |
|------|----------|---------|-----------|
| **None** | No uniforms updated | No discard | Full PBR, no occlusion |
| **ShaderBased** | Player + camera position only | Height + XZ discard | Ray-distance alpha + dither/blend |
| **RegionBased** | Interior flood-fill AABB | In-region discard | In-region discard |
| **Hybrid** *(default)* | Both | Both checks | Both checks |

## Transparency Techniques

| Technique | AlphaMode | Prepass | Main pass | Notes |
|-----------|-----------|---------|-----------|-------|
| **Dithered** *(default)* | `Mask(0.001)` | Binary height discard | Bayer 4×4 ordered dither | No MSAA cost; `MAY_DISCARD` enables prepass fragment shader |
| **AlphaBlend** | `AlphaToCoverage` | No discard | Sets `base_color.a`; hardware MSAA blends | Smooth edges; slight MSAA cost |

## Shader Files

| File | Pipeline stage | Purpose |
|------|---------------|---------|
| `assets/shaders/occlusion_material_prepass.wgsl` | Depth prepass fragment | Binary keep/discard; writes depth for below-player voxels only |
| `assets/shaders/occlusion_material.wgsl` | Main pass fragment | Full PBR + dither/blend occlusion logic |

## Key Design Decisions

- **Depth prepass uses binary discard, not dither.** Dithered discard in the prepass would write depth for ~50 % of above-player fragments, blocking the player in a checker pattern. The prepass discards everything above `player_y + height_threshold` within XZ radius.
- **`AlphaMode::Mask(0.001)` instead of `Opaque`.** `Opaque` doesn't set `MeshPipelineKey::MAY_DISCARD`, so Bevy skips the fragment stage in the depth prepass entirely — the custom prepass shader would never run.
- **Main pass re-writes depth for over-discarded fragments.** Voxels at the edge of the XZ zone that the prepass discarded conservatively will pass the `GreaterEqual` test against the cleared far-plane depth (0.0 in reverse-Z) and write their own depth correctly.
- **Shadow pass detection uses `view.clip_from_view[3][3]`.** The prepass shader runs for both the camera depth prepass and directional light shadow map passes. To skip height-discard during shadow rendering, the shader checks `view.clip_from_view[3][3] >= 0.5`: orthographic projection (shadow map) yields `[3][3] = 1.0`; perspective projection (camera) yields `[3][3] = 0.0`. In Bevy 0.18 the field was renamed from `view.projection` to `view.clip_from_view` — using the old name silently breaks the prepass pipeline and suppresses all shadow casting.
- **`MATERIAL_BIND_GROUP` macro for bind group index.** In Bevy 0.18 the material bind group moved from index 2 to index 3. Both shaders use `@group(#{MATERIAL_BIND_GROUP})` so Bevy injects the correct index at compile time, making the shaders forward-compatible.
