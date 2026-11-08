# `bevy_carnage::viscera`

XPBD Cosserat-style strands with a tearing mesenteric membrane: guts that spill out of a wound, fall with weight, coil on the floor, stay tethered by the mesentery, and then tear loose from it.

> **A module of `bevy_carnage` since 0.5.0.** This was a crate of its own — the family below — until 2026-09-04, when the seven leaves were folded back into the one crate a game is meant to depend on. This page is its module documentation, kept whole; the paths in it are spelled the way a consumer reaches them now.

![A blocked-out figure broken open; strands of gut spill from the torso, hang with weight, coil on the floor and tether the chunks that fell](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/viscera.gif)

## The family

This module is one of seven kernels `bevy_carnage` composes; the umbrella README lists them all with the path each is reached by. Nothing here is a separate dependency any more.

## The idea

A strand is a polyline of unit-mass nodes solved by **extended position-based dynamics**: each constraint carries a physical compliance `α` that enters the projection as `α̃ = α / Δt²`, so "near-inextensible bowel" and "limp mesentery" are two numbers rather than two solvers. A mesentery is a fan of compliant pins from fixed world points onto nodes of one strand — a 12 mm leash at the shipped defaults, sized so one link carries about nine nodes of hanging weight. Hang more than that off it and it parts — and **the tear is monotone, like clotting**: nothing in this crate ever clears the flag, so a tear cannot heal and the simulation cannot oscillate between held and free.

```rust
use bevy::math::Vec3;
use bevy_carnage::viscera::{spill, step, tube_mesh, Mesentery, ViscSettings};

let settings = ViscSettings::default();

// Four strands out of a wound, fanned around the exit direction. A pure function of the seed.
let mut strands = spill(Vec3::new(0.0, 1.4, 0.0), Vec3::new(0.3, 0.2, 1.0), 4, 0xA11CE, &settings);

// Tether each one back to where it left the body. One link carries about nine nodes, so anchoring
// every fourth node holds; anchor more sparsely than that and the membrane parts under the fall.
let mut mesentery: Vec<Mesentery> = strands
    .iter()
    .map(|s| {
        let anchors = s
            .nodes()
            .iter()
            .enumerate()
            .filter(|(n, _)| n % 4 == 0)
            .map(|(n, p)| (n as u32, *p))
            .collect();
        Mesentery { anchors, ..Default::default() }
    })
    .collect();

for _ in 0..600 {
    step(&mut strands, &mut mesentery, &settings);
}

// The render mesh is handed back. This crate never spawns.
if let Some(first) = strands.first() {
    let _mesh = tube_mesh(first, 8);
    println!("{:016x}", first.digest());
}
```

## Determinism is the product

- **Fixed substeps, fixed iterations, fixed sequence.** Four substeps of eight passes, projected stretch → bend → mesentery → floor, ascending index order throughout. No early-out on convergence: a strand that has already settled runs the same number of passes as one that is still falling, because a variable iteration count is a variable result.
- **No clock.** The tick length is the crate constant `FIXED_DT` (60 Hz), never `Time`. Bevy's `Time<Fixed>` defaults to **64 Hz**, so set `Time::<Fixed>::from_hz(60.0)` if you want viscera to fall at wall-clock speed. Reading the app's timestep would make the digest a function of a runtime setting.
- **No RNG.** `spill` derives every per-strand quantity from its `seed` through the crate's own integer bit-mixer.
- **ECS query order decides nothing, structurally.** A strand reads its own nodes, its own tether and the settings — never another strand. There is no shared accumulator, no budget and no last-writer-wins field, so two runs that visit the entities in opposite orders produce identical digests. That is why there is no `StrandOrder` component: a total order over nothing is still nothing.
- **`Strand::digest()`** is FNV-1a over the nodes' `f32::to_bits()`, in node order, little-endian by name rather than by host.

## What it deliberately does not do

**It never spawns.** `tube_mesh` hands back a `Mesh`; choosing the material, the handle and the parent is the caller's job. What the tube should look like is offered, not imposed: `gut_image()` hands back a texture for the tube's own `UV_0` (since 0.5.41) — a salmon-pink serosa with the mesentery's fat along the tethered side, faint straight vessels round the wall from it, the end that left the wound bloodied and smears dragged along the rest — to put in a material's base colour with a white factor, at `GUT_ROUGHNESS` (0.3). Drawn in the blood pool's own red, every spill read as red tubing lying in it; and a clean pink wall was as wrong the other way, paler than the blood it lay in.

**A spilled strand is lumpy, slumped where it lies and soaked in blood (0.8.6).** `tube_mesh` draws each ring at the strand's radius times two sines along its length (`LUMP_MAJOR`, `LUMP_MINOR`), their phases from `Strand::seed`, which `spill` sets per strand from its own seed (a strand laid by hand has seed `0`; the seed is not in `digest()` and nothing in the solver reads it). `soaked_tube(strand, sides, floor_y)` is the same mesh with two more things that need the floor: a ring that lies on it is squashed to `SLUMP_SQUASH` of its height and widened to keep its area, and every vertex carries `ATTRIBUTE_COLOR`, the tint of a venous blood film over the pale wall (`soak_tint`, from `bloodstain::spectral`'s Kubelka–Munk, so no blood colour is authored) at the node's `soak`: the coat the wound left (`exp(-arc / WOUND_REACH_M)`, restarting at any tear), contact with the floor, and a patchy base so the coat is thick in one place and a smear in the next. The material multiplies it into `gut_image()`. All of it is outside every digest.

**An organ is a lump of meat with a seed (0.8.7).** `organ_mesh(seed, size)` is a closed mesh of Gray's liver (*Anatomy of the Human Body*, 1918: 20 to 22.5 cm across, 15 to 17.5 deep, 10 to 12.5 thick, "a soft solid", "a dark reddish brown", specific gravity 1.05): a unit sphere displaced to a wedge of half-extents `ORGAN_HALF_EXTENTS` times `size`, thinning by `ORGAN_TAPER` toward its left end, lumped by three octaves of seeded noise (`ORGAN_LUMP`), and cut flat below by `ORGAN_FLAT` so it has a contact patch where it lies on the floor. Its lowest point is `y = 0`; its normals are the displaced surface's, found by differences of the displacement, so there is no seam and no flat shading; `UV_0` goes round the organ for `organ_image()`, a `128 x 64` texture of dark reddish brown mottled at the scale of a lobule with pale capsule patches, blood smears and deep patches. Both are pure functions, and like `tube_mesh` they spawn nothing: the preset throws `ORGANS_PER_SPILL` of them from a torso hit that opened muscle, as `Flying` pieces that lie where they land. One wedge with another seed stands for a stomach and a spleen too (`docs/research/organs.md`). Outside every digest.

**Strands do not collide with each other, or with anything but the floor.** Self-collision on a coiling rope is the expensive half of this problem and it needs a broadphase the caller almost certainly already owns.

**The solver knows nothing about blood.** A rope solver has no business importing a rheology crate: nothing in `solver`, `strand` or `spill` does. Only the cosmetic `look` module reads the colour of a blood film from `bloodstain::spectral`, for `soak_tint` (and, in any other `bloodstain::Fluid`, for `SoakTable::new(&fluid)`, which the preset builds from `WetSettings::fluid`).

## Papers

- Deul, Charrier & Bender, *Direct position-based solver for stiff rods*, Computer Graphics Forum 37(6) — `doi:10.1111/cgf.13326`. The XPBD compliance form and the stretch/bend split.
- Bergou, Wardetzky, Robinson, Audoly & Grinspun, *Discrete elastic rods*, ACM TOG 27(3) — `doi:10.1145/1399504.1360662`. Bending against a material frame; this crate uses a position-only surrogate instead, and says so at the constraint.
- Macklin, Müller & Chentanez, *XPBD: position-based simulation of compliant constrained dynamics*, MIG 2016. The `Δλ = (−C − α̃λ) / (Σ w|∇C|² + α̃)` update and the per-substep multiplier reset.


## What it exposes

**Plugin** — `VisceraPlugin`. Adds `ViscSettings` with `init_resource` and one system on `FixedUpdate`.

**System sets** — `VisceraSystems`. Everything the plugin runs is in it: order a mesh rebuild `.after(VisceraSystems)` and an anchor move `.before(VisceraSystems)`.

**Components** — `Strand` (private state; `Strand::new` lays a straight strand, `Strand::curved` one that turns segment by segment; `with_seed()`, `seed()`, `nodes()`, `radius()`, `digest()`), `Mesentery` (`anchors`, `tear_strain`, `torn`, all public).

**Resource** — `ViscSettings` (`substeps`, `iterations`, `gravity`, `damping`, `compliance_stretch`, `compliance_bend`, `floor_y`, `max_strands`).

**Functions** — `step`, `spill`, `tube_mesh`, `soaked_tube`, `soaked_tube_in`, `soak`, `soak_tint`, `gut_image`, `organ_mesh`, `organ_image`; and `SoakTable`, the tint of every soak of one fluid (`SoakTable::new(&Fluid)`, `.tint(soak)`; `soak_tint` and `soaked_tube` are its blood). All of them are plain functions over plain data and none of them needs an `App`. `spill` lays each strand draped and meandering rather than straight: it leaves the wound along its heading, turns toward the floor within about a quarter of its length, and hangs in a slow S, because a strand laid straight read as a rigid rod until gravity sagged it.

**Constants** — `FIXED_HZ`, `FIXED_DT`, `MAX_NODES`, `MAX_SEGMENTS`, `MAX_ANCHORS`, `MIN_REST_LEN`, `MIN_SIDES`, `MAX_SIDES`, `SPILL_SEGMENTS`, `SPILL_REST_LEN`, `SPILL_RADIUS`, `SPILL_CONE`, `STRAND_TEAR_STRAIN`, `DEFAULT_TEAR_STRAIN`, `COMPLIANCE_MESENTERY`, `GUT_ROUGHNESS`, `LUMP_MAJOR`, `LUMP_MAJOR_NODES`, `LUMP_MINOR`, `LUMP_MINOR_NODES`, `SLUMP_FADE`, `SLUMP_SQUASH`, `SOAK_BASE`, `SOAK_PATCH`, `SOAK_PATCH_M`, `WOUND_REACH_M`, `SOAK_FILM_MM`, `SOAK_STEPS`, `ORGAN_HALF_EXTENTS`, `ORGAN_TAPER`, `ORGAN_LUMP`, `ORGAN_FLAT`, and one `DEFAULT_*` per `ViscSettings` field.

## Examples

```sh
cargo run -p bevy_carnage --example rod_determinism   # terminal: 600 ticks twice, two digests
cargo run -p bevy_carnage --example spill             # windowed: press Space, guts fall and tear
```

`rod_determinism` is the one that proves the crate. It spills six strands, tethers three of them every fourth node and three every twelfth, steps 600 ticks, prints the digest, throws the state away, does the whole thing again, and prints whether the two match. No window, no `App`, no GPU — so it runs on a build machine. It prints the membrane split too, which is the other thing worth seeing: `0/21` links torn at the dense tethering, `9/9` at the sparse one.

`spill` is the one to watch. It spills on startup and again on every Space; each strand is tethered by a mesentery drawn as green lines, and a line turns into nothing at the moment its link's strain passes `tear_strain` — it never comes back. `R` clears. The counter in the corner is the live tear count, and the densely tethered strands hang from the torso while the sparse ones part and coil on the floor.

## Licence

MIT OR Apache-2.0, with the crate.
