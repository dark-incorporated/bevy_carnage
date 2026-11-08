# Determinism

Moved from the README on 2026-10-31, unchanged except for link paths, when the README was brought to the [org documentation guide](https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md). Its Rust blocks are still compiled as doctests (`src/lib.rs`).

Two runs of the same build, on the same asset, must produce bit-identical fragments. Two things make that true, and both were learned the hard way:

**The seed comes from the asset PATH, never its `AssetId`.** An `AssetId` is a slot index in the asset arena, handed out by async load order — so the same file gets a different id run to run, hashes to a different seed, and the mesh is partitioned along completely different planes. Measured, before the fix: 23 of 23 fragments differed, in half-extents as well as centres.

**The vertex soup is assembled in a canonical order.** `Children` order for a glTF scene is whatever order async instantiation happened to add nodes in. Fragment centroids are float sums over the merged soup, float addition is not associative, and so an unsorted soup moves every centroid by a few ULPs — same fragment count, positions off in the last bits. Sub-meshes are sorted by `(mesh asset path, world-matrix bits)` before a single vertex is appended, and the key is checked at runtime for uniqueness under `debug_assertions` or `strict-order`.

`hash_f32` is a hand-rolled integer hash with its output frozen in a test, for the same reason: there is no RNG dependency here, because a stream that may change between minor versions cannot underwrite any of the above.

**The preset's recorders check every canvas a skin owns.** `capture_preset` and `capture_skinned` end on one line, and two runs of a build must print the same one: `digest=` folds the wetmaps (and a holed cloth) as the clip ends, `flay=` every flaymap's depth buffer (the craters) and `dermis=` every `DermisCanvas` (the bruises and burns under the skin, `DermisCanvas::digest`), those two at every frame of the clip, because a body that breaks is despawned and nothing of its skin is left to fold at the end. Each is a sorted fold of per-canvas FNV-1a digests over the CPU buffers, never the uploaded images. Until `flay=` and `dermis=` existed a bruise or a crater could have come out differently from run to run and `digest=` would not have noticed; `capture_skinned --blows` shares the default's `digest=` and has its own `dermis=`.

Note what this does *not* claim. Fragment geometry is `f32` arithmetic, so cross-architecture bit-identity is not promised — only same-build, same-machine reproducibility, which is what a replay or a regression golden actually needs.
