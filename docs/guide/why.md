# Why: break the asset once, not the frame

Moved from the README on 2026-10-31, unchanged except for link paths, when the README was brought to the [org documentation guide](https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md). Its Rust blocks are still compiled as doctests (`src/lib.rs`).

The tempting version of this feature computes a fracture at the moment of impact. The shipped version, in more or less every game that has one, does not — it **pre-fractures the asset and swaps it in at runtime**, which is exactly what Müller, Chentanez & Kim document as the practical norm (ACM TOG 2013). Sellán et al.'s *Breaking Good* (ACM TOG 2022) is the most physically honest option available, and its fragments are precomputed **offline** through tetrahedralization and a conic solve in a Python/libigl toolchain — not something a minimal-dependency Rust crate can embed.

**But a static pre-fracture cannot answer where it was hit**, and Müller says so plainly: *"When a gamer shoots at a glass window, she expects the spider-web-shaped fracture pattern to be centered around the location where the bullet hit the glass. Anything else clearly destroys the illusion."* So the bake is not a fragment *set*. It is a hierarchy plus an adjacency graph, and every blow is a query against it. That is the shape PhysX Blast and Unreal's Chaos arrived at independently.

**The cut runs on a convex proxy, never on the triangle soup.** Müller's load-bearing observation is that `plane ∩ convex polyhedron = convex polygon`, which makes every cut face convex by construction and a centroid fan over it provably valid. There is no boundary-loop recovery in this crate because there is no input that needs one. The render triangles ride along as a payload, assigned to whichever cell bounds them and split only where they straddle a plane.

The cost of that choice is the proxy itself: **you supply it.** See "What it deliberately does not do".
