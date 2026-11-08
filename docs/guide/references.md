# References

Moved from the README on 2026-10-31, unchanged except for link paths, when the README was brought to the [org documentation guide](https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md). Its Rust blocks are still compiled as doctests (`src/lib.rs`).

- Müller, Chentanez & Kim, "Real-Time Dynamic Fracture with Volumetric Approximate Convex Decompositions", ACM TOG 32(4), 2013. DOI [10.1145/2461912.2461934](https://doi.org/10.1145/2461912.2461934)
- Sellán, Luong, Mattos Da Silva, Ramakrishnan, Yang & Jacobson, "Breaking Good: Fracture Modes for Realtime Destruction", ACM TOG 41(4), 2022. DOI [10.1145/3549540](https://doi.org/10.1145/3549540)
- Huang & Kanai, "DeepFracture: A Generative Approach for Predicting Brittle Fractures", 2023. arXiv [2310.13344](https://arxiv.org/abs/2310.13344) — Mott's fragment-size distribution
- Schvartzman & Otaduy, "Fracture Animation Based on High-Dimensional Voronoi Diagrams", I3D 2014.
- Sutherland & Hodgman, "Reentrant Polygon Clipping", CACM 17(1), 1974.
- [NVIDIA Blast](https://nvidia-omniverse.github.io/PhysX/blast/) — the chunk-hierarchy / support-graph / damage-shader vocabulary this crate's runtime half borrows.
