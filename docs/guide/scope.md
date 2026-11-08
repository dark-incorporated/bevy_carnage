# What it deliberately does not do

Moved from the README on 2026-10-31, unchanged except for link paths, when the README was brought to the [org documentation guide](https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md). Its Rust blocks are still compiled as doctests (`src/lib.rs`).

**It does not compute a convex decomposition.** You supply the proxy cells; the crate cuts them. A consumer already running V-HACD or CoACD for colliders has a decomposition, and forcing a second, different one would be the fracture disagreeing with the physics about what the object is. `ProxyCell::from_box` covers a blocked-out subject. Boring is not an exception to this: a `Bore` subtracts a prism *you* described from a cell *you* supplied, by plane splits with a closed-form decomposition, so what comes back is still a set of cells whose union is the shape you handed in — and a concave cell is still refused at the door.

**It does not move anything.** No rigid bodies, no velocities, no physics dependency. The bake hands you a mesh and a convex cell per piece; spawning them, building a collider and throwing them is your game's decision and your solver's job. `examples/explode.rs` integrates its own ballistics in thirty lines to make the point.

**It does not know what died.** No health, no factions, no damage types. It hands back a *reach* — geometry — and your game decides what an area is worth and what threshold means "this gives way".

**It does not own your schedule.** The plugin adds one system to `Update` in one public set. Whether that runs while a menu is open is yours to configure.

**It does not bond cells that merely touch.** Two cells are neighbours only when they share a *coplanar* face. V-HACD and CoACD produce cells that abut without their boundary polygons agreeing, so each root's subtree comes out as its own island unless your decomposition shares faces. Closing that with a proximity heuristic would silently weld a head to a torso.

**`fracture_mesh` fractures the bind pose.** Geometry is read from `Mesh3d` as authored, so a skinned character breaks from its rest pose, not its death pose. The preset carries each piece by the bone of the vertex nearest it, to where that bone is on the tick the body breaks (`0.5.28`), and `fracture_skeleton` (`0.7.0`) cuts a rigidly skinned body along its joints instead; a death-pose snapshot of a soft-skinned body is the proper upgrade, and per the fracture literature the gap is not visible when the chunks are flung fast.
