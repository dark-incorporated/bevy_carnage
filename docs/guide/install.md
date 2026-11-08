# Install, and the cargo features

Moved from the README on 2026-10-31, unchanged except for link paths, when the README was brought to the [org documentation guide](https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md). Its Rust blocks are still compiled as doctests (`src/lib.rs`).

```toml
[dependencies]
bevy_carnage = "0.9"
```

Requires **Bevy 0.20** and a Rust toolchain with **edition 2024**. That one line brings the whole family: `bevy_carnage::blood` (the material), `::wetmap`, `::viscera`, `::cross_section`, `::flaymap`, `::laceration` and `::fracture_modes` are the seven sibling crates re-exported whole, so nothing has to be named twice and no two versions of a leaf can meet in one graph.

`0.9` is on crates.io. The last release before `0.7.0`, `0.5.18`, is a Bevy 0.19 crate; a Bevy 0.19 project stays on `"0.5"`. To follow the repository instead of a release: `bevy_carnage = { git = "https://github.com/dark-incorporated/bevy_carnage" }`.

| feature | default | what it does |
|---|---|---|
| `serde` | ✅ | `Serialize`/`Deserialize` on the settings and hierarchy types, so a game can author dials in RON |
| `flesh` | ✅ | The flesh material and the drop-in preset — `bevy_carnage::flesh` and `::preset`. Pulls `bevy_image` and embeds a shader. Off, the deterministic half compiles without either, which is what a headless server wants |
| `strict-order` | — | Turns on the vertex-soup sort's runtime tie check in release. On automatically under `debug_assertions`; this is for a harness that builds in release and still wants it |
| `vfx` | — | GPU blood (`bevy_hanabi`) and stain decals. **Cannot be enabled on Bevy 0.20 yet**: `bevy_hanabi` has no release for it, so the six examples that need it are skipped by their `required-features`. The deterministic half — wounds, spatter, stains, bleed schedule, feel — is entirely outside it |
