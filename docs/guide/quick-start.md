# Quick start, in full: the preset and the bake by hand

Moved from the README on 2026-10-31, unchanged except for link paths, when the README was brought to the [org documentation guide](https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md). Its Rust blocks are still compiled as doctests (`src/lib.rs`).

## Quick start — the two-line inclusion

**0.4.0 adds a preset.** One plugin, a `Gore` on every surface that can be hurt (or one `GoreScene` on a loaded character), and a `GoreHit` per injury; everything below in this README is what the preset composes, and every one of its surfaces wears the flesh material. **Putting it on a real glTF character:** [`docs/preset.md`](../preset.md) is the checklist: the fixed step, `UV_0`, `uv_per_metre`, `gravity_uv`, the scale, the hit ray, and who owns the pieces.

```rust
use bevy::prelude::*;
use bevy_carnage::cross_section::Region;
use bevy_carnage::preset::{Gore, GoreHit, GorePlugin, GoreScene};

fn wire(app: &mut App) {
    app.add_plugins(GorePlugin);
}

fn dress(mut commands: Commands, torso: Entity, shirt: Entity, floor: Entity) {
    // Each mesh's atlas density is measured when it is dressed; state it only to override.
    commands.entity(torso).insert(Gore::skin(Region::Torso));
    commands.entity(shirt).insert(Gore::cloth());
    commands.entity(floor).insert(Gore::floor().with_uv_per_metre(0.3));
}

// Or dress a whole loaded character at once: put this on the entity its glTF scene is spawned under, and
// every mesh that appears below it, now or as the scene streams in, is dressed by the words in its node's
// name (`Head`, `Arm_L`, `shirt`). Rules you add win over `humanoid`'s own.
fn dress_scene(mut commands: Commands, character: Entity) {
    commands.entity(character).insert(GoreScene::humanoid().with("tabard", Gore::cloth()));
}

// `target` is a dressed mesh, or any ancestor of dressed meshes — the character your physics found.
// An ancestor is cast at every skin under it, as drawn, and the nearest is hurt.
fn on_raycast(mut hits: MessageWriter<GoreHit>, target: Entity, from: Vec3, dir: Vec3) {
    hits.write(GoreHit::impact(target, from, dir, 0.03, 12.0));   // a crater 3 cm wide, 12 mm deep
    hits.write(GoreHit::blunt(target, from, dir, 0.04));           // a bruise that ages
    hits.write(GoreHit::burn(target, from, dir, 0.03, 150.0, 1.0)); // a hot iron, one second
    hits.write(GoreHit::slash(target, from, dir, Vec3::X, 0.06, 8.0)); // a cut that gapes
}
```

What a hit does, in order: the flaymap peels to the depth asked; the wound's spatter lands on every `Gore::floor` (closed form, flown at a quarter of the paper's launch speed, `preset::GORE_SPATTER_SPEED_SCALE`, so it lands in the 4 m Comiskey, Yarin & Attinger measured; each droplet its own stain, and nine more for each one a floor takes alone, `GoreDials::floor_mist`, so a shot leaves a fan of drops: read as drops from a floor of 6 mm a texel, `GoreDials::floor_canvas` 512) and is cast at every other dressed surface; a bleed opens and pulses onto the wetmap until it clots, dripping a pool under the wound; a torso opened to muscle spills `bevy_viscera` strands; the first hit to show cortex forwards `BoneExposed` and — for a body that is a baked `FractureSubject`, or a runtime body carrying `GoreBreakable` — parts it along the fracture modes and throws the islands. A blow starts a `bloodstain::bruise` under intact skin that ages in time-lapse; a burn runs `bloodstain::burn` and reddens, blisters or chars to the necrosis depth; blood on cloth wicks along `bloodstain::wick`'s Lucas–Washburn front. `GoreDials` carries the numbers; `GoreClock` is the fixed tick every leaf is measured against.

**The flesh material** is what makes it read as tissue rather than as painted plastic: `bevy_carnage::flesh::FleshMaterial` is an `ExtendedMaterial<StandardMaterial, _>` with subsurface scattering pre-integrated per tissue from Jensen's dipole (`doi:10.1145/383259.383319`) after Penner's 2011 lookup, a wet clear coat driven by the wetmap's wetness byte, the blood film composited over the base from `bloodstain::spectral` so a fabric keeps its hue under a stain, and a dermis map for what lies under intact skin. Forward path, WebGL2-safe, one sampler; both of its tables are CPU-baked and frozen by a golden. `examples/flesh.rs` shows it alone and `examples/carnage_web.rs` — the flagship, which used to be a thousand lines of hand-wiring — is now the preset and nothing else.

**A skinned, animated character needs no extra step.** Put `Gore` on the skinned mesh as on any other. Hits, sprays and drips are cast at the pose the joints put it in, which is the pose on screen; a wound is a triangle and a place in it, so it bleeds from where that spot of skin is now; and the blood runs downhill in the world. A skin whose joints have no transform yet drops a hit with one warning rather than casting it at the bind pose. [`docs/preset.md`](../preset.md#skinned-characters) has the list, and `examples/skinned.rs` is the proof.

**A hit costs what its blood costs, not the world it lands in.** Twelve pellets from a shotgun in one tick take 3.1 ms on a character of 97 000 triangles, and one hit among 240 dressed surfaces 1.4 ms, on one Apple M5 in release. [`docs/preset.md`](../preset.md#what-it-costs) has the table and how to measure your own scene.

**The preset obeys the player's violence setting.** Insert a `GorePolicy` (`GorePolicy::for_tier(GoreTier::Blood)`) and the next hit follows it, so a settings menu is one resource: at `Stylised` a hit is still announced as `Wounded`, at the same point, and nothing bleeds, so the game puts a spark where the blood would have been; `Blood` keeps the blood and refuses the guts and the breaking; the world's share of the blood, the guts and the breaking are each a switch of their own. Reduction is substitution, never deletion.

![The same scripted scene three times: at the stylised tier a yellow spark marks each shot and nothing bleeds; at the blood tier the wounds bleed and pool on the floor and the body stays whole; at blood and gore the body comes apart](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/tiers.gif)

That is `capture_preset -- --tier 0|1|2`; `T` cycles it live in `examples/preset.rs`, and [`docs/preset.md`](../preset.md#the-players-settings) has the table.

## Quick start — the bake by hand

Mark what should break, then read the bake back when it dies. The launch is yours, and so is the solver.

```rust
use bevy::prelude::*;
use bevy_carnage::{CarnagePlugin, CarnageSystems, DetachedPart, FractureCache, FractureSubject};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState { #[default] Playing, Paused }

fn wire(app: &mut App) {
    app.add_plugins(CarnagePlugin)
        // The crate configures no run condition — when the bake runs is yours.
        .configure_sets(Update, CarnageSystems.run_if(in_state(GameState::Playing)));
}

// Mark what should break, and what should come off intact.
fn spawn_enemy(mut commands: Commands, assets: Res<AssetServer>) {
    let scene: Handle<WorldAsset> = assets.load("enemy.glb#Scene0");
    let enemy = commands.spawn((FractureSubject(scene.clone()), WorldAssetRoot(scene))).id();
    // ...once the scene streams in, tag whatever should detach intact:
    commands.entity(enemy).insert(DetachedPart);
}

// At the moment of death, read the bake at whatever granularity this death deserves.
fn on_death(cache: Res<FractureCache>, subject: &FractureSubject) {
    for frag in cache.leaves(subject.0.id()) {          // finest
        let _ = (&frag.outer_mesh, &frag.cap_mesh, frag.center_local, frag.half_extents);
    }
    for frag in cache.frontier_of(subject.0.id(), 3) {  // the same bake, as three big chunks
        let _ = frag.id;
    }
}
```

You do not need an `App` to use the fracture itself. `fracture_mesh` is the whole pipeline with no assets and no ECS — meshes in, meshes out:

```rust
use bevy::math::{Mat4, Vec3};
use bevy::shape::Cuboid;
use bevy::mesh::Mesh;
use bevy_carnage::{CutSettings, ProxyCell, fracture_mesh};

let body = Mesh::from(Cuboid::new(1.0, 2.0, 1.0));

// **The proxy is yours.** This crate cuts a convex decomposition and carries your triangles along
// as a payload — it never cuts the triangle soup. One cell per connected shell; a consumer already
// running V-HACD or CoACD for colliders has these, and a blocked-out subject can use `from_box`.
let proxy = vec![ProxyCell::from_box(Vec3::ZERO, Vec3::new(0.5, 1.0, 0.5))];

let cut = CutSettings::new(
    12,          // finest fragment count
    0.15,        // stop cutting below this fraction of the subject's size
    0xC0FFEE,    // seed — same seed, same pieces, every run
);
// Tier A — the convex cells — exists for every node the instant this returns. The drawn meshes are
// built for the nodes you ask for, which is why the accessors take `&mut`.
let mut baked = fracture_mesh(&[(&body, Mat4::IDENTITY)], &proxy, &cut);

assert_eq!(baked.frontier_of(3).len(), 3);   // one bake, read back as three pieces
let pieces = baked.leaves();                 // ...or as all of them
assert!(pieces.len() > 1);
assert!(pieces.iter().all(|p| p.outer.is_some() || p.cap.is_some()));
```
