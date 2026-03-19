# Occlusion Rendering Pipeline

Shows how each frame renders voxel chunks with occlusion transparency so the
player remains visible when walking inside buildings.

## Frame Pipeline

```mermaid
flowchart TD
    A([Frame Start]) --> B

    subgraph CPU ["CPU — Occlusion Update System"]
        B[Player & camera positions\ncollected from ECS]
        B --> C{Interior detection\nmode active?\nmode = RegionBased / Hybrid}
        C -- Yes --> D

        subgraph INTERIOR ["detect_interior_system"]
            D{Frame throttle:\nframes_since_update\n< region_update_interval?}
            D -- Skip --> SKIP([return — keep\nlast region])
            D -- Run --> D1{Player moved\n< 0.3 units since\nlast detection?}
            D1 -- Yes → no-op --> SKIP
            D1 -- No --> D2{SubVoxel entities\nadded or removed\nthis frame?}
            D2 -- Yes → spawn in progress --> D3[Set rebuild_pending = true\nreturn early]
            D2 -- No → settled --> D4{rebuild_pending\n== true?}
            D4 -- Yes --> D5[Clear occupied_voxels_cache\nrebuild_pending = false]
            D4 -- No --> D6
            D5 --> D6

            D6{occupied_voxels_cache\npresent?}
            D6 -- No --> D7["build_occupied_voxel_set\nIterate SpatialGrid cells\nSub-voxel bounds.min → floor → IVec3\nResult: HashSet of occupied voxel positions"]
            D7 --> D8
            D6 -- Yes → reuse cache --> D8

            D8["Check 4 XZ neighbour voxels the\nplayer overlaps (floor and floor+1\non both X and Z axes)"]
            D8 --> D9

            D9["find_ceiling_voxel_above (×4 positions)\nScan Y = player_y+2 … player_y+height_threshold\nFor each solid voxel found:\n  gap check — no solid between player and it?\n  Yes → ceiling found, return Y\n  No  → wall, keep scanning\nReturn None if nothing found"]
            D9 --> D10{Any ceiling\nfound in any\nof the 4 spots?}

            D10 -- Yes → best_ceiling = x,z,ceiling_y --> D11
            D10 -- No --> D12{Player still inside\nexisting region AABB\n(XZ only)?}
            D12 -- Yes --> D13[Keep current_region\nhysteresis — player near wall]
            D12 -- No --> D14[current_region = None\nPlayer is outside]

            D11["flood_fill_ceiling_region_voxel\nBFS queue, 4-connected XZ plane\nY search band: ceiling_y−1 … ceiling_y+2\nExpand to neighbour if current OR neighbour\nhas a ceiling voxel (crosses small gaps)\nCap: MAX_REGION_SIZE=1000 voxels\nMax search radius: 30 Manhattan steps\nTrack min_x, max_x, min_z, max_z"]
            D11 --> D15{voxel_count ≥ 2?}
            D15 -- Yes --> D16["Build InteriorRegion AABB\nmin.y = (player_y+2)−0.5  ← hide from 2 voxels above player\nmax.y = ceiling_y+100     ← hide everything above ceiling\nXZ = flood-fill extents ± 0.5 voxel padding"]
            D15 -- No → noise/single voxel --> D13
            D16 --> D17[current_region = Some(InteriorRegion)]
        end

        D17 --> E[InteriorState::current_region\nOption of AABB passed to shader]
        C -- No --> E2[region_max.w = 0\nregion inactive]
        E --> F
        E2 --> F
        F[Assemble OcclusionUniforms:\nplayer_pos, camera_pos,\nmode, technique, region bounds]
        F --> G[Upload uniform buffer\nto GPU via ExtendedMaterial]
    end

    G --> H

    subgraph PREPASS ["GPU — Depth Prepass\n(DepthPrepass component on camera)"]
        H[Vertex shader:\nTransform voxel chunk vertices\nWrite world_position]
        H --> I{technique == Dithered\nAND mode != None?}
        I -- Yes --> J{world_pos.y >\nplayer_y + height_threshold\nAND within XZ radius?}
        J -- Yes --> K([discard\nDon't write depth])
        J -- No --> L([Write depth\nFragment kept])
        I -- No --> L

        H --> M{mode == RegionBased\nOR Hybrid\nAND region active?}
        M -- in_interior_region → true --> K
        M -- outside region / inactive --> L
    end

    L --> N

    subgraph MAINPASS ["GPU — Main Forward Pass\n(AlphaMask3d render phase)"]
        N[Early-Z test vs prepass depth\nFragments above player → fail GreaterEqual\n→ skipped entirely]
        N --> O{Survive depth test?}
        O -- No → culled --> P([Fragment discarded\nby hardware])
        O -- Yes --> Q

        Q[pbr_input_from_standard_material\nSample shadow maps, lighting]
        Q --> R{mode == RegionBased\nOR Hybrid\nAND in_interior_region?}
        R -- Yes --> S([discard])
        R -- No --> T

        T{mode == ShaderBased\nOR Hybrid?}
        T -- Yes --> U[calculate_occlusion_alpha\nHeight + XZ ray distance\nsmooth falloff]
        U --> V{technique?}
        V -- Dithered --> W{dither_check\nBayer 4×4 matrix}
        W -- alpha ≤ threshold --> S
        W -- alpha > threshold --> X
        V -- AlphaBlend --> Y[Set base_color.a\nATC/blending]
        Y --> X
        T -- No / None --> X

        X[apply_pbr_lighting\nDirectional + point lights\nShadows]
        X --> Z[main_pass_post_lighting_processing\nFog, tonemapping]
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
