# Contributing

Moved from the README on 2026-10-31, unchanged except for link paths, when the README was brought to the [org documentation guide](https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md). Its Rust blocks are still compiled as doctests (`src/lib.rs`).

Work is done **in this repository**, which is the source of truth: open issues and pull requests here. [`Ladvien/foundation_vs_slop`](https://github.com/Ladvien/foundation_vs_slop) carries a frozen Bevy 0.19 copy (`0.5.64`) that is never mirrored over this one, so a change sent there does not reach this crate.

Before opening a PR:

```sh
cargo test                  # unit + tests/leaf.rs + doctests
cargo build --release       # NOT redundant with `cargo test` — see below
cargo build --examples
cargo clippy --all-targets  # 12 warnings pre-exist; add none. See BACKLOG.md's definition of done
```

**`cargo build --release` is load-bearing.** `cargo test` compiles dev-dependencies, and the dev-dependency here is the *full* `bevy` umbrella. Cargo unifies features, so under `cargo test` this crate silently gets every `bevy` feature there is and a missing entry in its own `[dependencies]` cannot fail. That is not hypothetical — `WorldAsset` was reached for while only `bevy_scene` was declared, and every test passed on a crate that did not build.

Two house rules worth knowing before you write code:

- **One path per feature.** No fallbacks, no legacy shims, no stub placeholders. Where the primary path cannot produce a usable result, it fails loudly. `seed_from_path` refusing to bake an unpathed handle *is* the rule.
- **This crate never learns what died.** No health, no factions, no damage, no physics. `tests/leaf.rs` enforces the dependency half of that; the naming half is on you.

Anything that changes emitted geometry regenerates the clips in `docs/` — see [docs/DEMOS.md](../DEMOS.md#regenerating-these).
