# The API

Moved from the README on 2026-10-31, unchanged except for link paths, when the README was brought to the [org documentation guide](https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md). Its Rust blocks are still compiled as doctests (`src/lib.rs`).

## The API

### Baking, in the ECS

| item | kind | what it is for |
|---|---|---|
| `CarnagePlugin` | `Plugin` | Registers the cache, the settings, and the bake |
| `CarnageSystems` | `SystemSet` | On `Update`. Gate and order against this, not the system |
| `FractureSubject(Handle<WorldAsset>)` | `Component` | What to break. The cache key *and* the seed source |
| `FractureProxy(Vec<ProxyCell>)` | `Component` | Your convex decomposition, subject-local. Required |
| `DetachedPart` | `Component` | A subtree pruned out and baked as one intact chunk — a carried weapon, a hat |
| `FractureSettings` | `Resource` | Twelve bake dials. `init_resource`d, so yours wins if inserted first |
| `bake_fractures` / `materialise_fragments` | fn | The two systems, chained. Public so they can be named in an ordering constraint |

### Reading a bake back

| item | what it is for |
|---|---|
| `FractureCache::leaves(source)` | The finest granularity — every fragment that was never cut further |
| `FractureCache::frontier_of(source, n)` | The same bake as roughly `n` pieces. **The granularity dial** |
| `FractureCache::at_depth(source, d)` | Every branch cut to the same level |
| `FractureCache::tree(source)` | The hierarchy itself: `FragmentTree`, `TreeNode`, `FragmentId` |
| `FractureCache::bonds(source)` | The `BondGraph` for the finest frontier |
| `FractureCache::detached_chunk(source)` / `is_baked(source)` | The pruned part; whether the bake has happened |
| `FractureCache::solids(source)` | Tier A for **every** node — the convex cells, present the instant the bake finishes |
| `FractureCache::request(source, ids)` / `ready(source, ids)` | Ask for a frontier's drawn meshes a frame ahead; check they arrived |
| `Fragment` | `id`, two mesh handles, the convex `cell`, `center_local`, `half_extents` |

### Where the blow landed

Each query returns a `Reach` — a severity in `[0, 1]` per bond — and **you** pick the threshold at which one gives way.

| item | models |
|---|---|
| `spread(graph, at, min, max)` | a projectile — nearest fragment, then outward **along the bonds** |
| `capsule(graph, a, b, min, max)` | a swung edge — falloff from the segment a blade travelled |
| `swept_triangle(graph, a, b, c)` | a swept blade — every bond the swing passed *through*, no falloff |
| `radial(graph, at, min, max)` | a blast — falloff from a point in open space |
| `directional(graph, at, dir, min, max)` | a pull — weighted by how squarely each face meets it |
| `Reach::above(threshold)` | the bonds that give way. Scale severity by material first if you like |
| `BondSet` | your accumulated damage. `sever_all`, `is_broken`, `severed` |
| `BondGraph::islands(members, broken)` | what is still connected. Stateless — call it after every blow |
| `BondGraph::of(members, capacity)` | build a graph for a frontier other than the finest |

### Geometry, with no ECS

| item | what it is for |
|---|---|
| `fracture_mesh(parts, proxy, &CutSettings)` | The whole pipeline. Meshes in, `Fracture` out |
| `Fracture` | `solids()` + `tree` + `bonds`, with `leaves()`, `frontier_of()`, `at_depth()` (all `&mut`: Tier B is built on request) |
| `CutSettings` | The geometry dials for one bake, without the ECS sizing policy |
| `ProxyCell` | One convex cell. `from_box`, `points()` (→ your collider), `volume()` (→ mass) |
| `audit_proxy` / `audit_render` / `SolidAudit` / `SurfaceReport` | Measure what the fracture produced |
| `hash_f32` | The frozen integer hash the fracture seeds from |

### Skinned meshes

Everything the preset casts a ray at — a hit, a droplet, a drip — lands on the body *as it is drawn*. `bevy_carnage::skin` is the CPU half of what `bevy_pbr`'s vertex shader does, so a game can ask the same question of an animated body.

| item | what it is for |
|---|---|
| `skin::Pose::of(&SkinnedMesh, &SkinnedMeshInverseBindposes, world)` | The joint matrices (`joint_global · inverse_bindpose`) at this instant. `None` while a joint has no transform yet or the lists disagree |
| `skin::Rig::of(&Mesh)` | A mesh's per-vertex joint indices and weights, borrowed. `None` when it is not skinned |
| `skin::posed_mesh(&Mesh, &Pose)` | The mesh as the pose draws it, world space, with `UV_0` and indices: cast it with `GlobalTransform::IDENTITY` |
| `skin::posed_vertex(&Mesh, &Pose, i)` | One vertex under the pose |
| `preset::SkinAnchor` | A spot on a skinned mesh as a triangle and a place in it, so it can be posed again each tick. Held by `WoundSite::skin` |
| `preset::RunDirection` | Which way blood runs on a skinned part right now, in UV space: downhill in the world at its wounds |

**Breaking a skinned body at its joints.** `fracture_mesh` cuts a convex proxy, and a body cut per bone comes out as islands, because a bone meets the next at a joint and not over a polygon. `fracture_skeleton` is the other way in: say which joints make one piece and where two pieces meet, and it cuts the body's meshes once, one piece per group, and declares the bonds, so the region queries and `BondGraph::islands` of the cell fracture work on a skeleton. It reads a **rigidly** skinned body only and refuses anything else by name, naming the part and the triangle or vertex, rather than cutting a piece that would jump at the break.

| item | what it is for |
|---|---|
| `fracture_skeleton(&[&Mesh], &[Mat4], &[JointGroup], &[JointLink])` | One piece per group, in the meshes' own bind space and not recentred, so a piece drawn with its joint's `joint_global · inverse_bind` lands where the body drew it. `Err(SkeletalRefusal)` when the body cannot be cut exactly |
| `JointGroup { joints, anchor }` / `JointLink { a, b, joint }` | The joints that make one piece and the one that carries it; two pieces and the joint whose bind position is where they are bonded |
| `SkeletalFracture { pieces, bonds }` / `SkeletalPiece { id, anchor, meshes, center }` | The pieces, and a `BondGraph` built by `BondGraph::declared` over `FragmentId(0)..FragmentId(n)` |
| `SkeletalRefusal` | `NotSkinned`, `NotIndexed`, `NotTriangleList` (a strip or a line mesh is not read as a list), `BadIndex`, `UngroupedJoint`, `SplitVertex` (a vertex weighted across two groups: the body is soft-skinned), `SplitTriangle`, `BadLink`, `MissingBind` |

### The carnage layer

All of it pure functions over a `Wound`, all of it available with `vfx` off. Nothing here reads a clock, spawns anything, or writes a `Transform`.

| item | what it is for |
|---|---|
| `Wound` / `WoundKind` | The one value everything downstream reads: `at`, `normal`, `area`, `severity`, and which of the two openings it was |
| `wounds_from_reach(graph, reach, threshold)` | The wounds a blow opened, sorted by `BondId` so the order is a function of the graph |
| `wounds_from_bonds(graph, broken)` | The wounds a set of already-severed bonds represents |
| `cap_faces(cell)` / `largest_cap(cell)` | Every raw-interior face of a convex cell, and the widest one — "the wound on this chunk" |
| `wound_of_channel(cell, exit, direction)` / `wound_from_ejecta(chunk)` | A bore's channel, as a wound. Its area is the channel *wall*, not the entry disc |
| `Wound::to_world(&GlobalTransform)` → `Wounded` | Subject-local to world. A normal goes through the affine's linear part, never `transform_point` |
| `Wounded` | The crate's one message. `CarnagePlugin` registers it; you write it |
| `droplet(w, i, s)` / `droplets` / `droplet_count` | The spray. Indexed, so any subset recomputes without the rest |
| `landing(from, &droplet, gravity, plane_y)` | Closed-form ballistic landing. `None` when it never crosses — no invented answer |
| `stains(w, s, plane_y)` / `Stain` / `stain_radius` | Where blood lands and how wide it reads. **Core, not cosmetic** |
| `Bleed` | `Component`. `opened_at` and `area`; everything else is derived from `tick - opened_at` |
| `pulses_on` / `flow` / `clotted` / `pulse_wound` / `pulse_period` | The heartbeat, the taper, and the clot — integer ticks throughout |
| `trauma_for` / `hitstop_ticks` / `shake_offset` | Impact feel, as numbers. **You** apply them |
| `CarnageSettings` | Eighteen carnage dials, `deny_unknown_fields` with a serde default per field |
| `BLOOD_DENSITY` / `BLOOD_SURFACE_TENSION` / `FORWARD_SPATTER_SPEED` / `BACK_SPATTER_SPEED` | The measured constants, each with its citation |

Behind `vfx` only: `CarnageVfxPlugin`, `CarnageVfxSystems`, `CarnageEffects` (its four wound bursts in `Bursts`, `blood` and `substitute`), `EffectTtl`, the five effect builders (the four wound bursts take a `Palette`), `DROPLET_STREAK_S`, `puff_image`, and `SplatTextures` / `spawn_stain` / `splat_image` / `build_splats` for stain decals. A camera rendering a stain **must** carry `DepthPrepass`.

### The dials

`FractureSettings` sizes the bake per asset; `CutSettings` is what a single cut actually needs.

| dial | default | effect |
|---|---|---|
| `pieces_base` / `ref_extent` / `min_pieces` / `max_pieces` | 14 / 0.5 / 6 / 40 | Fragment count, scaled by the mesh's own size |
| `min_fraction` | 0.18 | Stop cutting a piece below this fraction of the subject's size |
| `max_depth` | 12 | Cuts from a proxy cell to the finest fragment — the hierarchy's memory bound |
| `plane_jitter` | 0.35 | How far a cut plane slides off centre. `0.0` halves every piece and reads as uniform shards |
| `size_spread` | 0.5 | How much the largest-first cut order may be nudged, widening the size distribution |
| `weak_axis` | 0.75 | How hard to cut across a piece's *narrow* dimension — where a real thing comes apart |
| `cap_relief` | 0.30 | How much the drawn cut face is crumpled |
| `soften` | 0.5 | How much the drawn fragment is rounded. **Tier B only** — the collider never changes |

The last four are look dials, and the last two touch only the *drawn* mesh: the proxy cell stays exactly planar and convex, so colliders and every watertightness guarantee are untouched. `fracture_cube` prints the proof — cell volume identical at every `soften`, drawn area falling away.
