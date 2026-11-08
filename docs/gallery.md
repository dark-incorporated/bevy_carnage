# Gallery: the demos, recorded

Moved from the README on 2026-10-31, unchanged except for link paths, when the README was brought to the [org documentation guide](https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md). 

**[docs/DEMOS.md](DEMOS.md) — the examples, with recordings**, what each is for, and how to regenerate the clips. The two 0.4.0 clips:

| | |
|---|---|
| ![flesh](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/flesh.gif) | ![preset](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/preset.gif) |
| `capture_flesh` — the material: a banded limb, a skin sphere, a sheet, under a walking light | `capture_preset` — the preset: a blow, a burn, a slash, and shots to the bone |

![A smooth mannequin walks in place with its left arm raised and waving; a shot opens a wound on the raised forearm, a slash opens a thigh, shots open the chest, and each wound keeps bleeding from the limb it is on while blood pools under the feet](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/skinned.gif)

That is `examples/skinned.rs` — **the preset on a skinned, animated body**, made in code with no asset. A game's character is a mesh whose asset is its bind pose, arms at its sides, and a skeleton that carries it somewhere else each frame; before 0.5.19 the preset cast every hit at the asset, and a shot at the raised forearm above did nothing. Now the hit lands where the arm is *drawn*, the wound rides the limb it opened on, and its blood runs downhill in the world rather than along the atlas. Every key aims at where that part of the body is this frame; `capture_skinned` records it.

![A mannequin walks in place with its left arm raised; a heavy shot to the chest breaks it mid-stride into pieces that keep the pose, the raised arm up and the legs mid-step, before they fall and settle among its guts](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/skinned_break.gif)

That is `capture_skinned --break`, and **a body that breaks breaks where its skeleton is.** The fracture is baked from a mesh asset, which is the bind pose; before 0.5.28 the pieces of a character caught mid-stride appeared with its arms at its sides. Each piece is now carried by the bone of the vertex nearest it, to where that bone is on the tick the body breaks and turned as it is. The one thing asked of the game is that a skinned body's fracture is baked in the mesh's own space.

![The mannequin walks in a pale sleeveless shirt; two shots to the chest punch a ragged hole in the shirt through which the wound is seen, and blood soaks outward through the cotton along a frayed, pinkish front](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/skinned_shirt.gif)

That is `capture_skinned --shirt`: **a shot through clothing holes it.** The shirt is a seventh skinned mesh named `Shirt`, which `GoreScene` dresses as cloth. Before 0.5.29 the shot wounded the skin under it unseen and the shirt took a flat red patch; now the impact opens a hole where its ray crossed the shirt, cut from the colour, the shadows and the depth prepass alike, and the blood wicks into the weave around it.

![The mannequin walks in front of a plaster wall; a shot through the raised forearm and two through the chest throw a cone of fine spatter onto the wall behind it, each drop its own small stain](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/skinned_through.gif)

That is `capture_skinned --through`: **a shot that goes through.** Whether a bullet leaves a body is the game's ballistics, so the game says so, with `GoreHit::through()`. The preset goes through everything of the body on the shot's line, opens an exit wherever it comes out, wider than the way in, bleeds from every wound, and throws each exit's forward spatter along the shot. The wall is `Gore::wall()`, a hard upright surface on which every droplet is its own stain, sized, stretched and pointed by the way it struck.

![Three crops of the floor in front of the mannequin after the clip's three shots: 0.7.0 at 256 texels, five blobs of blood; the same floor now, a fan of about forty small stains; and the floor at 512 texels, about fifty finer ones](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/media/floor_spray.png)

That is `capture_skinned --eye 0.0,1.5,3.1 --at 0,0,1.3` (frame 359, labelled, the right panel with `--floor 512`): **a floor takes a wound's spray as drops.** Until the release after 0.7.0 a floor stamped a droplet as the blob a sheet takes, so a shot left a few pieces of jam; now each droplet is its own stain, sized and slanted by how it landed, and `GoreDials::floor_mist` throws nine fine droplets more at the floors for each one it throws at everything. A wall, a skin and a cloth take what they took. What the picture also shows, and what the next release answers: on a three-metre slab at the shipped 256 texels, 12 mm a texel, a drop is a few texels and reads as a small rounded square. That was put down to the canvas's resolution; it was the canvas's *data*, and the next picture is the same two shots after it was fixed.

![Four crops of the same two shots: the floor at the shipped 256 texels and the wall of --through, each as 0.8.1 drew it and as the next release draws it; the floor's drops go from rounded squares of about four centimetres with a pale middle to small round drops of mixed sizes, and the wall's from near-identical red beads to a fine spatter](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/media/drops.png)

That is `capture_skinned --eye 0.0,1.5,3.1 --at 0,0,1.3` (top) and `capture_skinned --through` (bottom), frame 359, 0.8.1 on the left and `WetCanvas::paint_cone` on the right: **a drop is drawn at the size it is.** A droplet's stain under eight texels across is the cone its edge makes (the canvas holds `DETAIL_EDGE − CONE_SLOPE · distance`, which the flesh shader's cubic B-spline reproduces exactly, so the line it draws at 4 % is the ellipse): round, at its own size, at its own place to a fraction of a texel, and it holds still while it is wet, where a blob of blood diffused over the next hundred ticks into the diamond a four-neighbourhood makes. Nothing in the shader changed.

![The mannequin stands with its right forearm held across its chest; one shot goes through the forearm and on into the chest, and as the arm is lowered the chest wound it hid is seen bleeding, the wall behind spattered by the chest's exit](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/skinned_guard.gif)

That is `capture_skinned --guard`: **a shot through a forearm goes on into the chest.** The body stands with its right forearm across its chest, and one shot through it goes in and out of the forearm and then in and out of the chest. When the arm comes down the wound it was hiding is there, the forearm's forward spatter round it, and the chest's exit has spattered the wall. Until 0.5.62 the shot stopped at the far side of the forearm and the chest took a sheet of blood with no wound under it.

![Slowed four times: one shot through the forearm held across the mannequin's chest throws a fan of thin red streaks from both wounds, which slow and shorten as they fly, and a soft red mist hangs where the shot went in](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/skinned_spray.gif)

That is the same shot at a quarter speed, `capture_skinned --guard --slow 4`: **the spray is streaks.** Every droplet is drawn along its velocity, as long as a ninetieth of a second of its flight, so it leaves the wound as a streak and shortens as drag slows it; at full speed it is out of a body-sized view within two frames. The stains on the wall behind are already there in the first frame: the preset paints a stain the tick its wound opens, ahead of the spray that carries it (see the known limits in [`docs/preset.md`](preset.md)).

![The mannequin walks across a tiled floor through a spill of blood and out the other side, leaving a line of bloody boot prints, left and right, each paler and more broken than the last](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/skinned_stroll.gif)

That is `capture_skinned --stroll`, on a floor of 30 cm tiles whose UVs repeat as a level's do (`Gore::projected`): **a foot that stepped in blood tracks it.** When a foot lands is the game's animation, so the game says so, with the `GoreStep` it raises for a footstep's sound. A sole in wet blood picks some up and leaves a sole-shaped dent; a sole carrying blood prints a boot's outline on the floor and sheds 30 % of what it carries, so the prints fade from a whole sole to a scatter. Dried blood is not picked up.

The seven kernels have a clip each, on their own pages under `docs/`, and every one was cut from a headless recorder in this crate's `examples/` — `capture_gore` for the four cut-face kernels, `capture_family` for the blood pair, `capture_entrails` for the strands — so the clip you see is a script two runs must reproduce digest-for-digest:

| | | |
|---|---|---|
| [![cross-section](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/cross_section.gif)](cross_section.md) | [![fracture modes](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/fracture_modes.gif)](fracture_modes.md) | [![flaymap](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/flaymap.gif)](flaymap.md) |
| [![laceration](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/laceration.gif)](laceration.md) | [![spectral blood](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/spectral.gif)](bloodstain.md) | [![wetmap](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/wetmap.gif)](wetmap.md) |
| [![viscera](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/viscera.gif)](viscera.md) | | |

![A blue blocked-out humanoid standing; a projectile takes off its arm, another its head, a slash takes the other arm, a blade through the waist takes both legs and a blast finishes the torso](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/sever.gif)

That is `examples/sever.rs`, on a fixed script. The subject **stays standing** between blows and what comes off depends on where you hit it. Run it and you aim it yourself:

```sh
cargo run --release --example carnage          # needs a GPU
cargo run --release --example sever            # needs a GPU
cargo run --release --example explode          # needs a GPU
cargo run --release --example bullet_holes     # needs a GPU
cargo run --release --example ribbons          # needs a GPU — a blood strand per flying chunk
cargo run --release --example entrails         # needs a GPU — torso chunks pull their guts out
cargo run --release --example fracture_cube    # terminal only — no window, no GPU
cargo run --release --example skinned          # needs a GPU — the preset on an animated, skinned character
```

```text
  arrows / WASD   move the aim marker
  1 … 5           projectile · slash · swept blade · blast · pull
  G               granularity — cycle which frontier of the bake is standing
  T               soften — cycle how hard the drawn fragments are rounded
  R               reset
```

![A blue blocked-out humanoid standing still while five shots punch through it one at a time, each leaving a small dark-red hole in the blue skin and throwing a handful of rounded red chunks out the far side that arc down, land and spread into overlapping dark pools on the floor, then the camera orbits a third of a turn to show the wider exit wounds and the spatter together](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/holes.gif)

That is `examples/bullet_holes.rs`, on a fixed script. Each shot is a `Bore` — a segment, a radius and three look dials — subtracted from the proxy before any cut, so the hole has a wall rather than being painted on a surface, and the plug it removed is thrown out the exit side as a chunk of gore that lands and becomes a flat stain. The subject keeps standing because the shards around a channel share their radial faces bit-for-bit and the bond graph reads them as one island. Run it and you aim it yourself, with `[`/`]` for calibre, `J` for raggedness, `F` for exit flare and `K` for how many pieces the plug breaks into.

![A blue blocked-out humanoid; a shot punches a channel through its chest which mists blood from the hole, then four blows in turn take an arm, the head, the other arm and both legs, each cut throwing a red spray outward along the face it opened while dark stains pile up on the floor beneath; the severed pieces keep pulsing blood at a heartbeat's rate as they lie there, the pulses weaken, and by the end the floor is soaked and the bleeding has stopped](https://raw.githubusercontent.com/dark-incorporated/bevy_carnage/main/docs/carnage.gif)

That is `examples/carnage.rs`, on a fixed script — and it is the one clip that shows the whole crate at once. **A bullet hole and a severance are geometrically different openings, and they bleed through the same code**: frame 18 is a bore, the rest are severances, and nothing downstream can tell them apart beyond a `WoundKind` mixed into a seed. The spray leaves each cut along that cut's own normal, the floor stains are solved on the CPU and would exist with the render feature off, and every severed piece keeps pulsing at its own heartbeat until it clots. Run it and you aim it yourself, with `1`–`5` for the five blows and `6` to shoot a channel through.

Its recorder, `capture_carnage`, prints one line that two runs must agree on — `carnage: frames=… wounds=… stains=… digest=…` — which is the determinism check for the whole layer: the bake, the bond graph, wound extraction and its canonical sort, the wound seed, the droplet draws, the ballistic solve and the pulse schedule. A digest that moves means something read a clock, an `Entity`, or an unsorted iteration order.
