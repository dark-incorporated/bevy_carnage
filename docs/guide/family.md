# The family: one crate, seven kernels

Moved from the README on 2026-10-31, unchanged except for link paths, when the README was brought to the [org documentation guide](https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md). Its Rust blocks are still compiled as doctests (`src/lib.rs`).

This crate is the whole gore stack. **Depend on it alone**: every kernel below is a module of it, reached by the path in the last column, and there is nothing else to add — no second dependency line, no version pin to keep aligned with this one. Each was a published crate of its own until 0.5.0 (the links are the archived repositories, kept for their history); a game never needed seven, so they came home.

| module | what it is | reach it as |
|---|---|---|
| the umbrella | plane cuts with watertight caps, bores, energy-driven fracture, wounds, decals, impact feel, the flesh material and the preset | `bevy_carnage` |
| [`bloodstain`](../bloodstain.md) | blood as a material: Carreau–Yasuda rheology, Comiskey spatter, stain morphology, drying, spectral colour by thickness and oxygenation, and the three injury kernels (bruise, burn, wick) — engine-free, `[f32; 3]` in every signature | `bevy_carnage::bloodstain` (also `::blood`) |
| [`wetmap`](../wetmap.md) | texture-space blood that runs, spreads and dries — CPU-authoritative, so a canvas can be hashed | `bevy_carnage::wetmap` |
| [`viscera`](../viscera.md) | XPBD strands with a tearing mesentery: guts that spill, coil, tether and tear | `bevy_carnage::viscera` |
| [`cross_section`](../cross_section.md) | anatomical bands on a cut face from a sourced per-region thickness table, via `UV_1` | `bevy_carnage::cross_section` |
| [`flaymap`](../flaymap.md) | texture-space flaying: skin, fat, muscle, cortex peel under hits, with a once-per-canvas bone handoff | `bevy_carnage::flaymap` |
| [`laceration`](../laceration.md) | a cut that gapes on a time curve, scaled by skin tension and Langer-line orientation | `bevy_carnage::laceration` |
| [`fracture_modes`](../fracture_modes.md) | Sellán's fracture modes on a cell graph: a fixed-schedule bake, impact projection, gluing partition | `bevy_carnage::fracture_modes` |

Every module is deterministic where it can be — fixed schedules, no clocks, frozen digests over its CPU state. `bloodstain` has no engine in it at all, and `tests/leaf.rs` keeps it that way now that it is a module rather than a crate boundary. Fourteen of the family's examples run in a browser at [ladvien.github.io/foundation_vs_slop](https://ladvien.github.io/foundation_vs_slop/), built from the monorepo's frozen `0.5.64` copy on Bevy 0.19.
