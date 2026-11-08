# bevy_carnage

> ⚠️ **Vibe Coded.** Written by AI agents working from a human's direction, covered by tests rather than a line-by-line human review. Read the code before you rely on it for something you care about.

Wounds that peel to the bone, bleed, pool and dry, and bodies that break into chunks, for Bevy characters, deterministic to the bit.

[![crates.io](https://img.shields.io/crates/v/bevy_carnage.svg)](https://crates.io/crates/bevy_carnage) [![docs.rs](https://img.shields.io/docsrs/bevy_carnage)](https://docs.rs/bevy_carnage) [![CI](https://github.com/dark-incorporated/bevy_carnage/actions/workflows/ci.yml/badge.svg)](https://github.com/dark-incorporated/bevy_carnage/actions/workflows/ci.yml) [![Bevy 0.20](https://img.shields.io/badge/bevy-0.20-purple.svg)](https://bevyengine.org) [![licence](https://img.shields.io/badge/licence-MIT%20OR%20Apache--2.0-blue.svg)](#who-and-licence)

| | |
|---|---|
| ![A textured blocked-out body takes a blow, a burn, a slash and shots to the chest; the wounds bleed and pool on the floor and the body comes apart](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/preset.gif) | ![A smooth mannequin walks in place with its left arm raised; a shot opens the raised forearm, a slash the thigh, shots the chest, and each wound keeps bleeding from the limb it is on](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/skinned.gif) |
| `examples/preset.rs`: a blow, a burn, a slash and shots to the bone | `examples/skinned.rs`: the same on a skinned, animated body, the wounds riding the limbs |

**Status:** usable · Bevy 0.20

**Why:** a fracture computed at the moment of impact costs a frame and a static pre-fracture cannot know where the shot landed, so this crate bakes each asset once into a hierarchy and a bond graph, and every blow is a cheap query against it, which is the shape PhysX Blast and Unreal's Chaos arrived at too ([why](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/guide/why.md)).

## Quick start

```toml
[dependencies]
bevy_carnage = "0.9"
```

```rust
use bevy::prelude::*;
use bevy_carnage::preset::{Gore, GoreHit, GorePlugin, GoreScene};

fn wire(app: &mut App) {
    app.add_plugins(GorePlugin);
}

// Dress a loaded character: every mesh under it is dressed by its node's name (`Head`, `Arm_L`).
fn dress(mut commands: Commands, character: Entity, floor: Entity) {
    commands.entity(character).insert(GoreScene::humanoid());
    commands.entity(floor).insert(Gore::floor());
}

// One message per injury, along your hit ray.
fn on_hit(mut hits: MessageWriter<GoreHit>, target: Entity, from: Vec3, dir: Vec3) {
    hits.write(GoreHit::impact(target, from, dir, 0.03, 12.0)); // a crater 3 cm wide, 12 mm deep
    hits.write(GoreHit::slash(target, from, dir, Vec3::X, 0.06, 8.0)); // a cut that gapes
}
```

Putting it on a real glTF character has a checklist: [`docs/preset.md`](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/preset.md). The full quick start, and baking a fracture by hand, are in [the guide](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/guide/quick-start.md).

## Try it

```sh
cargo run --release --example preset
```

The left GIF above, with keys to aim it yourself; `T` cycles the violence setting.

![The preset mid-scene: the body has come apart on the floor into chunks with raw red cut faces, lying in a pool of blood beside a standing sheet](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/media/preset.png)

*A frame of `preset` after the last shot: the chunks, their cut faces, and the pool.*

## What you can do

- **Hurt a character with one message**: `GoreHit::impact`, `blunt`, `burn` and `slash` peel skin, fat and muscle to bone, bruise and burn under the skin, and cut wounds that gape; a wound has a pale lip and, from a hand's width, a ragged margin round a knobbly floor that glints ([0.8.7 and now](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/wound.png)); a shot sent `through()` leaves a round entrance and a torn, star-shaped exit ([the pair, 0.8.3 and now](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/exit.png)): [quick start](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/guide/quick-start.md).
- **Make wounds bleed where they are**: a pulsing bleed runs downhill on the pose on screen, pools on the floor and dries from its edge in where it fell (a dark crust round a red middle, crazed once dry: [the front, 0.8.4 and now](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/drying_front.png)), a wall's spatter keeping every drop it landed as, on skinned, animated bodies too: [`docs/preset.md`](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/preset.md).
- **Let an artery spurt**: `GoreHit::arterial()` makes the wound throw an arc of large drops on every heartbeat, flying the parabola to the walls and floor, each stain on a wall with its run and a stipple of fine drops beside it, from where the wound is and the way it faces as the body turns, until it clots ([the clip, 0.8.9 beside now](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/hose.gif), [the stills](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/hose.png)): [`docs/preset.md`](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/preset.md#arterial-wounds).
- **Cut meat that reads as meat**: the muscle in a cut face and in a laceration's bed is a cherry red divided by pale sheets between its bellies and by wandering marbling, drawn at the sizes the anatomy has them and tuned against measured beef and photographs of a shank, not a flat burgundy with a hairline net ([0.8.10 beside now](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/cutface.png), [the clip](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/cutface.gif)): [`docs/cross_section.md`](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/cross_section.md).
- **Cut faces that glisten**: the cut faces of a body that has broken are no longer flat plates of one tone but wet, undulating surfaces whose clear coat catches glints along their folds, with the colour unchanged ([0.8.12 beside now](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/relief.gif), [the stills](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/relief.png)): [`docs/preset.md`](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/preset.md#what-you-get-back).
- **Blood that is not red**: `WetSettings { fluid: Fluid::Green, .. }` (or `Ichor`, a black oil, `Blue`, `Milk`, a white, or a `Fluid::Custom(Pigment)` of your own absorption bands) makes every surface the preset draws as blood that fluid, from the same film model as red blood and not a swatch: wounds and their runs, pools, a wall's spray, a shirt's soak, a cut face, the guts' soak, each aged to a crust of its own, with the same spray, wounds and digest to the bit ([the clip, one hit in five fluids](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/fluids.gif), [the stills](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/fluids.png)): [`docs/preset.md`](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/preset.md#blood-that-is-not-red).
- **Burst a body**: `GoreBurst` throws a body's blood in every direction at once, a spray of 960 droplets on the one size-and-speed law a wound's cone obeys, each flying the parabola to the first wall, floor, ceiling, cloth or body it meets, with the body shadowing what is behind it and the stains arriving over three quarters of a second at a bounded cost a tick ([0.8.11 beside now](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/burst.gif), [the stills](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/burst.png)): [`docs/preset.md`](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/preset.md#bursts).
- **Throw a spray that reads as drops**: each droplet a wound throws lands as its own round stain at the size it is, on a floor, a wall or a skin, and holds still while it is wet: [the gallery](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/gallery.md).
- **Drag a hand, a body or a sleeve through blood**: `GoreSmear` lifts the wet blood a contact crosses and lays what it carries along its path in parallel streaks, heavy where it touched down and broken into lines as it runs dry, on a floor or a wall, the blood moved and none made ([the clip](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/smear.gif), [the stills](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/smear.png)): [`docs/preset.md`](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/preset.md).
- **Break a body into its own pieces**: bake once, then shoot a shoulder and the arm comes off, with watertight cut faces and convex colliders, and the guts it spills lie soaked, lumpy and flattened in its pool, two dark organs among the loops ([0.8.6 and now](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/media/organs.png)): [the API](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/guide/api.md).
- **Bore bullet channels that come out the far side**: the hole and the gore it throws are the same subtraction: [features](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/guide/features.md).
- **Honour the player's violence setting**: `GorePolicy` swaps blood for sparks, or keeps blood and refuses the breaking, by substitution and never deletion: [`docs/preset.md`](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/preset.md#the-players-settings).
- **Replay a fight exactly**: every canvas is owned by the CPU and hashed, so two runs of a build print the same digest: [determinism](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/guide/determinism.md).
- **Reach the seven kernels underneath**: blood rheology, wetmaps, viscera, cross-sections, flaying, laceration and fracture modes, all modules of this one crate: [the family](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/guide/family.md).

## Part of Dark Inc.

`bevy_carnage` is one of the [Dark Inc.](https://github.com/dark-incorporated) crates. The Dark Factory, Dark Inc.'s editor, uses it for destructible props and its damage dials: [use it in the Dark Factory](https://dark-incorporated.github.io/crates/bevy_carnage.html#use-it-in-the-dark-factory).

## Bevy compatibility

| `bevy_carnage` | Bevy |
|---|---|
| 0.6 – 0.8 | 0.20 |
| 0.5 | 0.19 |

## Learn more

- [docs.rs/bevy_carnage](https://docs.rs/bevy_carnage): the API, with this page as its front page.
- [The book page](https://dark-incorporated.github.io/crates/bevy_carnage.html), and [the gallery](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/gallery.md) and [`docs/DEMOS.md`](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/DEMOS.md) for every recorded example.
- [`docs/guide/`](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/guide/README.md): everything the old README carried, from the API to what each release changed.
- [`examples/`](https://github.com/dark-incorporated/bevy_carnage/tree/main/examples) and [`CHANGELOG.md`](https://github.com/dark-incorporated/bevy_carnage/blob/main/CHANGELOG.md).

## Who and licence

Maintained by [@Ladvien](https://github.com/Ladvien). This repository is the source of truth: issues and pull requests belong at [github.com/dark-incorporated/bevy_carnage/issues](https://github.com/dark-incorporated/bevy_carnage/issues), and [how to contribute](https://github.com/dark-incorporated/bevy_carnage/blob/main/docs/guide/contributing.md) follows the org's [CONTRIBUTING.md](https://github.com/dark-incorporated/.github/blob/main/CONTRIBUTING.md).

MIT OR Apache-2.0, at your option.
