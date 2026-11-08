# Changelog

All notable changes to this crate are recorded here. `0.6.0` and `0.7.0` (Bevy 0.20, `fracture_skeleton`) are
described in the comment block at the top of `Cargo.toml`; they and every entry below were published together as
`0.7.0`, the first release since `0.5.18`, which is a Bevy 0.19 crate.

## [Unreleased]

Meets the family's machine plate (`dark-incorporated/.github/docs/STYLE.md`):

- Every system is in `CarnageSystems`: `GoreSystems`, `FleshSystems`, `CrossSectionSystems`, `FlaymapSystems`, `LacerationSystems`, `VisceraSystems`, `WetmapSystems` and (with `vfx`) `CarnageVfxSystems` are configured into it in each schedule they run in, and the preset's `retube` is in it directly. The `vfx` startup bake `build_effects` is now in `CarnageVfxSystems`.
- The settings resources (`FractureSettings`, `CarnageSettings`, `CrossSectionSettings`, `FlaySettings`, `ModeSettings`, `ViscSettings`, `WetSettings`) derive `Reflect` with `reflect(Resource, Default)` and, with `serde`, `reflect(Serialize, Deserialize)`; `CrossSectionSettings`, `ViscSettings` and `WetSettings` gain serde derives (`#[serde(default)]`) and `Scale` gains them too. `CarnageSettings::blood` and `WetSettings::fluid` are `#[reflect(ignore)]` (their types are in the engine-free `bloodstain` module): serde still saves them, a reflected write leaves them as they are.
- Every public component derives `Reflect` with `reflect(Component)` and is registered by the plugin that owns it. Fields of engine-free or opaque types are `#[reflect(ignore)]`.
- `tests/machine_plate.rs`.

## [0.9.0] - 2026-11-02

### Added

- **Blood that is not red** (iteration 20, AG-160, `docs/research/fluids.md`). The preset drew blood or, at `GoreTier::Stylised`, nothing, and a game that needs its blood not red (a rating board's "animated blood", a robot's oil, an alien, *Evil Dead II*'s goo) had to lose the blood or fork the crate. `bloodstain::Fluid` (also `bevy_carnage::Fluid`) is **what the blood is**: `Blood` (the default, the Bosschaart table and the haemoglobin age walk, to the bit what it has always been), `Ichor` (black oil: melanin's power law, thin amber-brown, thick black, stays glossy), `Green` (biliverdin), `Blue` (oxyhaemocyanin), `Milk` (scattering and almost no absorption, which whitens what it lies on), and `Custom(Pigment)`: a `Pigment` is a flat absorption, a power-law continuum, up to three Gaussian `Band`s, a flat reduced scattering, and how it dries (`dried`, a per-channel multiplier, and `dries`, how far it sets). A fluid is a different `μa(λ)` and `μs'` in the **same Kubelka-Munk film model** (`Fluid::srgb`, `linear_srgb`, `reflectance`, `lightness`, `key`), never a swatch. `dry::appearance_in(&Fluid, age, hz, area, &BloodSettings, fresh)` is `appearance_with_fresh` for blood and the pigment's own walk (monotone, bounded, no serum halo, bending at blood's knot) for any other fluid. Every number a named pigment carries is this crate's own, and says so.
- **`WetSettings::fluid`** is the one dial. A canvas's film, its runs, pools and ageing (`WetCanvas` re-tabulates its colours when it changes), the flesh material's blood table (`flesh::luts(&Fluid)`, baked from it on `Startup`, so a cut face's film, a cloth's soak and a crater's floor follow), and a spilled strand's soak (`viscera::SoakTable::new(&Fluid)`, `soaked_tube_in`, rebuilt by the preset's `retube` when the fluid changes) all read it. **A fluid is a colour and nothing else**: every recorder's digest is 0.8.13's and equal under every fluid (`capture_skinned` `5f8eb5573aea933f`, `--hose` `0c903766bf51f0f2`, `--break` `f89e1d5e3bcfc119`, `--shirt` `579e84b9d3ccfe51`), a canvas stamped under two fluids has one `digest()`, and `bench_carnage` reads 0.09467 ms a frame and 2.714 ms p99 against 0.09496 and 2.727. No shader changes: the flesh material reads the same texture, with other bytes in it, so the WebGL2 budget is untouched.
- `capture_skinned --fluid blood|ichor|green|blue|milk` and `capture_preset --fluid`; `docs/media/fluids.gif` and `fluids.png`; `docs/DEMOS.md` and `docs/preset.md` ("Blood that is not red"). Fifteen tests: the film's hues and directions (green is green, thin ichor is brown, an absorber darkens and a scatterer whitens), the ageing, the canvas, the tables, the soak, the preset end to end, and blood's bit-for-bit equality with the model it was.

### Changed

- **Breaking** (a MINOR release, as the crate is below 1.0): `WetSettings` has a new public field, `fluid`; `flesh::blood_lut`, `aged_blood_lut` and `luts` take a `&Fluid` (`&Fluid::Blood` is what they were); `flesh::lut_digest` and `aged_lut_digest` are blood's and their goldens stand. `viscera::soak_tint` and `soaked_tube` remain, in blood.
- Install lines read `bevy_carnage = "0.9"`.

## [0.8.13] - 2026-11-02

### Changed

- **A cut face is a wet surface, not a plane** (iteration 19, AG-159, `docs/research/cut-face-relief.md`). Every cap the preset draws (the cut faces of a body that has broken, `FleshMode::Cap` with a film) was a flat polygon of one tone, lit by two suns and an environment: one shade and no highlight, the "flat red sail". `flesh.wesl` now lifts it by a slow undulation, a two-octave value noise over the face's planar `UV_0` (so it stays with the piece as it falls) `CAP_BUMP_MM` (16 mm) across and `CAP_RELIEF_MM` (4 mm) high, as a normal from `lifted_normal`, and lifts its **clear coat's normal by `COAT_BUMP` (3) times the slope the tissue's takes** with the coat `CAP_COAT` (1.8) times a crater wall's: a liquid's surface tilts further than the folds it lies on, and at the tissue's own slope the coat caught no highlight however strong it was drawn. **No colour changes.** Measured on the same pixels of `capture_preset --eye 0.30,0.42,0.62 --at 0.0,0.10,0.0` frame 300 (the geometry and the digest `04758d82ec10aa5b` are 0.8.12's): inside the cut faces the local variation goes from 0.61 to 1.65 (2.7 times), the luminance's spread from 7.65 to 10.0, 2.6 % of their pixels are 15 or more brighter than they were (0.9 % by 30), the mean colour `(128.8, 49.0, 55.9)` to `(128.8, 51.3, 57.8)`. Four rounds on the amplitude and the scale, the coat and the coat's normal; the values and what each drew are in the shader's comments and the note. Eight value-noise evaluations a fragment on a cut face, none elsewhere, no binding, texture or uniform: `tools/webgl2/check.sh` OK on big. **Nothing else moves**: every recorder's digest is 0.8.12's, twice each, `bench_carnage`'s is `0x4366a8b64352c85a` (the shader is not hashed and the CPU does not change). A PATCH: no public name or signature changes (the Rust diff is documentation).
- **What it replaced, and why the colour is as it was** (`docs/research/film-per-tissue.md`). The item began as the first of iteration 18's backlog, a film of blood per tissue so that the muscle strip of 0.8.11 shows through the preset's 1 mm film. It failed twice and neither attempt ships: a thin film per tissue (a value-noise mottle, a cellular net, hard lumps, a soft mottle at shares from 0.012 to 0.035 of 1 mm) made every face that shows its tissue read as food (cheese, coral, salmon), and a face that **drains** from the old film to the thin one (`FleshParams::with_drained`, a registry and a system in the preset, `cut_face_drained`, two tests, sixteen material writes a body) did the same, and clotting it (the dried-blood table at 0.12 to 0.4) made brick and terracotta. Both are parked, with their numbers, on `loop/iter-19-film` and `loop/iter-19-drain`. What `blood_lut()` says about what a film hides is in the note: the green and blue of what lies under a venous film are 3 and 4 % of what they are bare at 0.05 mm, and the red 61 % at 0.05, 37 % at 0.14, 17 % at 0.3 and 5 % at 0.6, so "opaque past about 0.15 mm" (this crate's own comments until now) was true of two channels and not of the third.

### Added

- `tools/webgl2/check.sh` drives **Firefox on an Xvfb display** when the host has no Chrome or no Node 22 (`WEBGL2_DRIVER=firefox` forces it): Mesa's llvmpipe gives it a WebGL2 context where headless Firefox has none (AG-109), now a script. It sends no input, so `CLICKS` is ignored there. On big (Linux, 24 cores) a cold wasm build of `carnage_web` is **3 min 24 s** and a rebuilt shader's check 50 to 72 s, not the 38 to 79 minutes recorded for a Mac.
- `capture_preset --eye x,y,z --at x,y,z`, the recorder looking at a heap of pieces close, and `capture_flesh --film`, the limb strip under the preset's film of blood: the test beds for a change to what a cut face looks like. Neither changes a digest.

## [0.8.12] - 2026-11-02

### Added

- **A body that bursts throws its blood in every direction** (iteration 18, AG-157, `docs/research/burst.md`). A wound's spray is a cone about its normal, so a head shot from the front stained what was in front and nothing round it. `preset::GoreBurst` (a `#[non_exhaustive]` message: `new(at)`, `.with_radius(m)`, `.with_droplets(n)`) is the cloud, sent **with** the `GoreHit` that breaks the body (it opens no crater and breaks nothing itself): `bloodstain::burst_spatter` draws `BURST_DROPLETS` (960) droplets uniformly on the sphere, on `droplet()`'s own size-speed law (Comiskey, Yarin & Attinger 2018) with the sizes biased toward the fine (`BURST_SIZE_BIAS`, 2.0), capped at `BURST_BUDGET` (2) times `max_droplets_per_wound`; each **flies the parabola** at `BloodSettings::gravity` to **the first dressed surface it crosses** (wall, floor, ceiling, cloth, the part of a body in the way, which shadows what is behind it; a back face within `BURST_SHELL` (2) radii of the centre does not catch, so a head does not stain its own inside) and is drawn `BURST_DROP_SCALE` (3) times the generator's size by `hard_stain`. **The stains arrive when their flight ends**, not on the tick they leave: each is queued on its surface and drawn at most `BURST_LANDINGS_PER_TICK` (24) a surface a tick (drawn at once they cost 9 ms on one tick for 190; deferred, the 630 to 740 of `capture_skinned --burst` land over 44 ticks at 0.2 to 2.5 ms a tick). A tier that draws no blood or no decals throws nothing, and `GoreReset` takes a surface's stains out of the air. Measured: the wall behind a 1.2 m body took 0.050 of an open wall's blood, 177 stains of 2 to 29 texels on a wall 1 m off, the cloud's centre of mass 10 cm or more lower under gravity than without. Ten tests; five mutations each fail one (`BURST_SHELL` 0, a skin that lets it through, no gravity in the throw, no size bias, a hemisphere). New public: `GoreBurst`, `BURST_DROPLETS`, `BURST_DROP_SCALE`, `BURST_RADIUS_M`, `BURST_SHELL`, `BURST_LANDINGS_PER_TICK` in `preset`, and `burst_spatter`, `BURST_BUDGET` and `BURST_SIZE_BIAS` in `bloodstain::patterns`, `burst_spatter` re-exported from `bloodstain`. **Nothing moves**: every existing recorder digest is 0.8.11's, twice each (`capture_skinned`'s eleven other variants, `capture_preset`, `capture_entrails`, `capture_flesh` and `capture_gore`), and `bench_carnage`'s digest `0x4366a8b64352c85a` is unchanged (`carnage_frame_ms` 0.09258 to 0.09314, +0.6 %, `carnage_p99_ms` 2.645 to 2.674). **It is the residue of a cloud and not the cloud**: nothing hangs in the air (a particle effect; `vfx` does not build on Bevy 0.20), there are no strings or chunks (the break's pieces are those), and the stains are the canvases' crimson, not the film's orange. `capture_skinned --burst` (`--wall-z`, `--break`, `--no-cloud`) is the demo; `docs/media/burst.gif` and `burst.png` set 0.8.11 beside now. A PATCH: additions only, no public name or signature changes.

## [0.8.11] - 2026-11-01

### Changed

- **A cut muscle reads as meat** (iteration 17, AG-156, `docs/research/cut-face.md`). The muscle band of the cross-section strip was a 0.3 mm film of venous blood over mid-grey: a purplish burgundy (mean sRGB `(123, 37, 48)`, L\* 29.2, below every value the beef
  literature reports) with a hairline white net of 3 mm cells and nothing larger, so a cut face read as mud or leather. Now: **the lean is a 0.05 mm film over a pale substrate** (`MUSCLE_FILM_MM`, `MUSCLE_SUBSTRATE`: sRGB `(170, 54, 56)`, the cherry of
  oxymyoglobin against a shank photograph's `(183, 39, 39)`), with **pale connective tissue at three scales**, the ones a cut muscle has (Biga et al., *Anatomy & Physiology 2e*, 10.2): sheets of epimysium between bellies 12 mm across, curved by a slow
  warp, swelling and thinning along their length, each belly its own tone (`BELLY_*`, `SHEET_*`); wandering lines of perimysium between bundles, a warped Worley field broken up by noise so a line fades in and out, tapering (`MARBLE_*`); and the 0.7 mm
  fascicle net, as before, fainter. Measured on the limb strip against windows set from measured beef (Crump et al. 2025: L\* 43-44, a\* 22; Lu et al. 2025: L\* 30-33, a\* 14-16) and a photographed shank: **mean L\* 43.4 (window 38-52), 10.6 % pale
  texels (8-20 %), the spread of the lightness of 6 mm blocks 3.6 (0.42 before)**, linear red share 0.76. Fat, skin, cortex and marrow are byte for byte what they were; the strips' digests move **on purpose** (`the_strips_are_frozen`, re-blessed:
  `[0x13b0398ae1709ac6, 0xe803b5652b6625b0, 0x602f16842b352486]`, was `[0x31620f0b7e9e7e88, 0x7066b93f54d6cc2c, 0x7d5d385f1cdc1df0]`), two tests hold the band to its windows and to its structure (mutation-checked: no structure, the old
  colour and a wrong noise period each fail), and **no recorder's digest moves** (none folds a colour; every one is 0.8.10's, twice each, and `bench_carnage`'s is `0x4366a8b64352c85a`, `carnage_frame_ms` 0.09366 to 0.09425, +0.6 % at a higher load, `carnage_p99_ms` 2.661
  to 2.701). The strips bake in 0.07-0.10 s at startup (0.05-0.07 s before). **It shows where a material does not hide it:** a plain atlas material, a laceration's bed, the flesh material's unfilmed `Cap` mode. **It does not show in the preset's cut faces
  or its craters**, which wear a 1 mm film of blood (`CUT_FACE_FILM_MM`), opaque past about 0.15 mm: the crater close-ups differ in 0.08-0.15 % of their pixels. A per-tissue film is a shader change and is not made. `docs/cross_section.gif` and
  `docs/laceration.gif` are regenerated; `docs/media/cutface.png` and `cutface.gif` set 0.8.10 beside now. A PATCH: no public name or signature changes.

### Added

- `capture_flesh --eye x,y,z --at x,y,z`: the flesh recorder looks from another place, close on the limb's strip or a crater; the digest depends on neither.

## [0.8.10] - 2026-11-01

### Added

- **An arterial wound throws an arc a heartbeat** (iteration 16, AG-155, `docs/research/hose.md`). `GoreHit::arterial()` set the blood's saturation and nothing else, and `bloodstain::arterial_arc` (one arc of large droplets a systole, falling with
  the pressure) had a generator, tests and documentation and no caller: an arterial wound bled as a venous one did. Now **the wound spurts**: on each beat of its schedule (`spurt_bpm` 96, one every 38 ticks at 60 Hz, until the clot at six
  seconds) `preset::spurt` throws `arterial_arc`'s droplets from where the wound is on the pose on screen and along the way it faces now (the winding normal of the triangle it opened on, so a wound on a waving forearm or a turning head sweeps
  its arc), each **flying the parabola** at `BloodSettings::gravity` (walked as chords of `HOSE_CHORD_M`, cast through the mesh index) onto the first wall or floor it meets and drawn as the stain `hard_stain` makes of it **as it arrives**, at
  `HOSE_DROP_SCALE` (3.5) times the generator's diameter. **On a wall a stain runs** (a narrow stamp drawn down from it, `HOSE_RUN_WIDTH_M`, `HOSE_RUN_STAINS`, `HOSE_RUN_MAX_M`) and **three fine drops fall beside each large one**
  (`HOSE_SATELLITES`, `HOSE_SATELLITE_CONE`). New public: `Spurts` and `SpurtSite` (a component beside `Wounds`, which a `GoreReset` clears: `WoundSite` is a public struct a field would break) and the `HOSE_*` constants. Walls and floors only: a
  jet that meets a body or a cloth passes through, and a cut face does not spurt. A tier that draws no blood or no decals throws nothing. `take_hits`'s mesh index is now a resource (`MeshIndex`) the new system shares and not a local. **Nothing else
  moves**: every existing recorder digest is 0.8.9's, twice each, and `bench_carnage`'s digest `0x4366a8b64352c85a` is unchanged (`carnage_frame_ms` 0.09360 to 0.09437, +0.8 %, `carnage_p99_ms` 2.672 to 2.676); `capture_skinned --hose` prints
  its own, `0c903766bf51f0f2`, and `--hose --vein` (the same hit, venous) `38099dce0d339ebc`. A beat costs about a tenth of a millisecond for 28 droplets against a floor and three walls and a tick without one nothing (the system runs only
  while some body has a `Spurts`). Not done, named: the curve of the references' stains (a beat's droplets leave on one tick), the gloss, the jet in flight, a jet that meets a body. Additive API: a patch.

## [0.8.9] - 2026-11-01

### Added

- **Smears: a hand, a body or a sleeve drawn through blood** (iteration 15, AG-154, `docs/research/smear.md`). Nothing drew a smear: a sole walking through a pool printed soles, and a hand slid down a wall or a
  body dragged out of a pool left only the blood that was already there. Now: **`WetCanvas::smear_uv(&Smear, carry, full, tick, settings) -> carry`** (`wetmap`) lifts `SMEAR_LIFT` (35 %) of the wet blood under a
  contact (dry blood does not move), carries it, and lays it along the path in streaks parallel to the motion, heavy where the stroke touched down (each line at its own distance, a ragged head) and broken into
  lines and dots as the load runs low, **conserving every byte** (`sum(amount) + carry` before and after, a test over random strokes); **`GoreSmear`** (`preset`: `carrier`, `from`, `to`, `normal`, `width_m`,
  `load_m2`, builders `new`, `width`, `loaded`) draws one on whichever `Gore::floor` or `Gore::wall` is under the contact, and the carrier keeps what it holds as **`Smeared`** in square metres of film, so a hand
  crossing from a wall to a floor takes the same blood; a tier that draws no blood smears nothing; `GoreReset` wipes a `Smeared`; new public `Smear`, `SMEAR_LIFT`, `SMEAR_PEAK`, `SMEAR_BEAD_GRAINS`,
  `SMEAR_FULL_M2`, `SMEAR_LOAD_MAX_M2`, `SMEAR_WIDTH_M`, `SMEAR_GRAIN_M`, `SMEAR_GAP_TICKS`. Measured as the references are (`smear_stat`, `capture_skinned --smear`, frame 245): gradient coherence 0.57, 0.46 and
  0.51 against the references' 0.56 to 0.61, streaks along the axis (alignment 1.00), the hands' last third 0.54 and 0.57 of the first (target at most 0.6), the dragged body's 0.76 (it bleeds as it goes).
  **Every existing digest is 0.8.8's** (`capture_preset`, every `capture_skinned` variant, `capture_entrails`, `capture_flesh`, `bench_carnage` `0x4366a8b64352c85a`), twice each; `--smear` prints its own,
  `b3f7357ae81ff12c`, twice. A segment costs 1 to 41 microseconds on a 512 canvas; `bench_carnage` with nothing smearing 0.0931 to 0.0954 ms a frame (the plugin runs one more system). Not done, named: a soaked
  hand's head is a slab where the references show lines (the film is opaque from about a third of full coverage), a smear does not sag, a skinned body is not smeared, no handprint. Additive API: a patch.

## [0.8.8] - 2026-11-01

### Changed

- **A wound has a margin and a floor** (iteration 14, AG-153, `docs/research/wound-margin.md`). The crater was a smooth bowl: a falloff over the whole radius whose rim had no slope, so a 30 mm torso crater was
  57 % of its radius of pale, orange skin and fat round a red floor with no features (`capture_gore` frame 260, reddish pixels `(134, 48, 47)`; **texture contrast `hp3` 0.057, where six wound-makeup
  photographs measure 0.29 to 0.37**). Now: **the profile has a wall and a pit** (`FlayCanvas::paint_uv` and `paint_uv_torn`: the depth rises over the outer 45 % of the radius and a third of it keeps deepening
  to the centre; the pale ring is 31 % of the radius, 46 % narrower, and the pit keeps a stack of hits a crater); **a crater of eight texels' radius or more has its rim out of round** (three harmonics, from a hash of
  the place; a bullet's hole stays round); and `shade` draws **the floor's lumps** (nodules and crevices at 16 mm over a 256 canvas, standing up in the normal image, paler and brighter and wetter on the
  nodule, darker and rougher in the crevice, fading out where the cortex starts and where a texel is bigger than half a lump) and **the skin's lip** (the first 3 mm of depth, paler than the skin round it and raised).
  Measured: the crater's `hp3` 0.057 to 0.236 and `hp6` 0.081 to 0.266; the preset's own close-ups (`capture_skinned --through`) entry 0.082 to 0.157 and exit 0.155 to 0.205, with the wound floor glinting where it
  had one soft highlight. **A deliberate model change**: `the_scripted_wound_is_frozen` is re-blessed (`0x46cc_108d_9766_f0be` to `0xecf0_7283_e699_8576`) and every recorder's `flay=` moves (`capture_preset`,
  `capture_skinned` default, `--blows`, `--break`, `--shirt`, `--through`, `--guard`; not `--stroll` or `--quiet`, which peel nothing), and `capture_flesh`'s digest with it (`d3150b10f27bcdd5` to
  `a869a0d72d0862e4`); every recorder's `digest=`, `capture_entrails` and `bench_carnage` print what they printed in 0.8.7, twice each. `shade` costs 19 to 29 % more (a big hit's 2.9 ms is 3.5 ms; a bullet's
  0.10 ms is 0.13 ms); no API changes; `the_strips_are_frozen` and `texel_at` are untouched. **What it does not do, named**: the nodules are red (under the blood film their cream tint hardly shows: the crater's
  red minus green is 80 where the references' is 86 to 109), there is no big dark pit, no thick rolled lip and no highlights beyond what the light gives (the 95th percentile of luminance is 105 against 134 to 165),
  no ribs and no flaps; a wound under a few texels across has few lumps; the finer-than-a-texel floor is `flesh.wesl`'s, which needs a WebGL2 re-check this pass's disk could not hold.

## [0.8.7] - 2026-11-01

### Added

- **`viscera::organ_mesh(seed, size)`, `organ_image()`, `ORGAN_HALF_EXTENTS`, `ORGAN_TAPER`, `ORGAN_LUMP`, `ORGAN_FLAT`; `preset::Organ`, `ORGANS_PER_SPILL`, `ORGAN_ROUGHNESS`.** An organ is a closed, wedge-shaped, lumpy mesh of Gray's liver
  (*Anatomy of the Human Body*, 1918: 20 to 22.5 cm across, 15 to 17.5 deep, 10 to 12.5 thick, "a soft solid", "a dark reddish brown") with a contact patch where it lies, its lowest point at `y = 0`, seamless normals found by
  differences of the displacement and `UV_0` round the organ for `organ_image()`; both are pure functions of their arguments and spawn nothing. Cosmetic: outside every digest.

### Changed

- **A spill has lumps in it** (iteration 13, AG-152, `docs/research/organs.md`). A torso hit that reaches muscle spilled loops and nothing else, where the *Day of the Dead* opened torso (Ref D2: reddish pixels
  mean sRGB `(127, 38, 40)`, red minus green 89) is a mass with dark lumps in it. It now also throws `ORGANS_PER_SPILL` (2) organs, 0.9 to 1.3 times Gray's liver, sideways by their number, at 0.12 to 0.42 m/s, each a `Flying`
  piece that falls to `GoreDials::floor_y` and lies there, in a `FleshMaterial` of the guts' muscle profile under `organ_image()` at roughness 0.35; `GoreDials::strands` at 0 and a viscera policy that refuses guts refuse them
  too, and there is no new `GoreDials` field. Measured on the pixels that differ between 0.8.6 and now (`capture_preset` frame 300, 993 of them): mean sRGB `(123, 41, 40)`, red minus green 81, luminance 58.6 (D2: 56.9),
  98th percentile 127.7 (D2: 133). **No digest moves**: `capture_preset`, `capture_entrails`, `capture_flesh` and all eight `capture_skinned` variants print what they printed in 0.8.6, twice each; `bench_carnage`'s digest is
  0.8.6's and its frame time within its noise (0.0933 ms against 0.0932, p99 2.70 against 2.67). **What it does not do, named**: a liver is thirty pixels at the preset's camera and reads as a dark wet lump, the organs
  have no contact shadow, and a stomach and a spleen are the same wedge with another seed. Slime strings were built, measured and **parked** (`docs/research/slime-strings.md`, branch `loop/iter-13`).

## [0.8.6] - 2026-11-01

### Added

- **`viscera::soaked_tube(strand, sides, floor_y)`, `soak`, `soak_tint`, `Strand::with_seed` and `seed`**, and their constants (`LUMP_MAJOR`, `LUMP_MAJOR_NODES`,
  `LUMP_MINOR`, `LUMP_MINOR_NODES`, `SLUMP_FADE`, `SLUMP_SQUASH`, `SOAK_BASE`, `SOAK_PATCH`, `SOAK_PATCH_M`, `WOUND_REACH_M`, `SOAK_FILM_MM`, `SOAK_STEPS`). `soaked_tube` is
  `tube_mesh` slumped where the strand lies on the floor and carrying `ATTRIBUTE_COLOR`: the tint of a venous blood film over the pale wall at each node's `soak`
  (`bloodstain::spectral`'s Kubelka–Munk, so no blood colour is authored). Cosmetic: the strand's nodes, `Strand::digest`, every canvas and every recorder's digest are untouched.

### Changed

- **Spilled guts are bloodied, lumpy and slumped on the floor** (iteration 12, AG-151, `docs/research/guts.md`). The preset's guts were clean salmon-pink tubes of one radius,
  paler than the pool they fell in: `capture_preset` frame 130, the reddish pixels of the hanging strands, mean sRGB `(126, 68, 66)`, red minus green 58; the *Day of the Dead* gut pull
  (pig guts and blood from a slaughterhouse, by Savini's account in the trivia) is `(138, 36, 31)`, red minus green 102. A strand is now drawn by `soaked_tube`: the coat the wound left
  (`exp(-arc / 0.30 m)`, restarting at a tear), contact with the floor and a patchy base give each node a soak, the tint is the venous film of `0.35 mm * soak^3`, and the material multiplies
  it into `gut_image`; `tube_mesh` (and so `soaked_tube`) draws each ring at the strand's radius times two sines along its length, their phases the strand's own seed, which `spill` sets per
  strand; a ring that lies on the floor is squashed to 0.72 of its height and widened to keep its area. Now: `(117, 36, 35)`, red minus green 80, and a strand in the pool is darker than
  the pool. **No digest moves**: `capture_preset`, `capture_entrails` and all eight `capture_skinned` variants print what they printed in 0.8.5, twice each; `bench_carnage`'s digest is
  0.8.5's and its frame time within its noise (the retube of five strands is 14.2 us with lumps and 16.8 us soaked, `cargo test --release`, a throwaway). `tube_mesh` keeps its signature and now lumps its radius.
  **What it does not do, named**: the highlight is not sharper (`GUT_ROUGHNESS` 0.3, 0.15 and 0.10 measured the same under the preset's camera, and the flesh shader draws a coat only from a wet canvas), the heap of
  a pile is three or four strands, there are no organ blobs, and a strand's blood does not dry (a strand has no canvas and the viscera module no clock).

## [0.8.5] - 2026-11-01

### Changed

- **Blood dries from its edge in, and a dried crust cracks** (iteration 11, AG-150, `docs/research/drying-front.md`). The flesh material drew a
  film's colour at the one age its stamp carries (`flesh::blood_dried`), and a pool or a run is stamped in one tick, so a pool dried the way a decal
  changes colour: `capture_skinned --through --floor 512 --dry 120`, frame 359, 0.8.4: a pool, a run and a wall's drops each one flat tone, edge to
  edge. A pool of blood dries at its contact line first and the front goes in (Smith, Nicloux & Brutin 2020, `doi:10.1038/s41598-020-65465-4`: "the rim
  dries first. Then this drying front propagates towards the centre of the pool"): a black rim round a red middle, and once dry a crust crazed
  into plates. `flesh.wesl` now reads the film's own extent in the wet map, as `crater_light` reads a crater's (six directions, seven taps each, out to
  30 mm on the surface), and draws an aged film at the age its distance from the edge gives it: the rim leads, the middle follows at 0.4 of the clock
  until the front reaches it, and at an age of one every part is at the end of the CPU's colour walk, as it was. The dry rim is matte while the middle
  still shines; the outermost 12 mm of a deep film is darkened and drawn at the film's full depth (the coffee ring); a dried film of a pool's depth is
  crazed in patches into 12 mm plates, 0.35 mm wide, showing the surface the blood lies on, faded out where plates are under twelve pixels across and gone under two, so
  they are a close-range detail. **No digest moves**: every `capture_skinned` variant and `capture_preset` print what they printed in 0.8.4, twice each; no
  table, canvas, uniform or binding is added (`the_flesh_tables_are_frozen` and `the_aged_blood_table_is_frozen` pass untouched), `bench_carnage`'s
  instructions retired are 0.8.4's, and the WebGL2 check (`tools/webgl2/check.sh`) prints `WEBGL2 PIPELINE OK`. Fresh blood costs nothing
  more; a dried-blood pixel costs 42 more taps of the wet map and, where it cracks, a lattice. **What it does not do, named**: a dried drop (a film under a fifth of the canvas's depth) is
  still one dark tone, with no ring and no crack, and the cracks are plates, not the radial cracks of the reference pool.

## [0.8.4] - 2026-11-01

### Added

- **`FlayCanvas::paint_uv_torn` and `paint_world_torn`** (iteration 10, AG-149, `docs/research/torn-exits.md`): the crater of `paint_uv` and
  `paint_world` with a margin torn the way an exit wound is, for a `tear` in `[0, 1]`. Over zero, the rim is moved out of round by two low
  harmonics of the angle and five to seven thin splits are laid over the crater, each at its own angle and length (a few short, most
  long), bent and rippled, tapering to a point, at `TEAR_SPLIT_DEPTH` (0.5) of the crater's depth; a `tear` of zero or NaN is `paint_uv` byte for
  byte. Every number is a hash of the centre texel and the angles go through `libm`: the same hit in the same place tears the same way on
  every run. New public constants `flaymap::TEAR_REACH` (1.3), `TEAR_SPLIT_DEPTH` (0.5) and `TEAR_SPLIT_WIDTH` (2.0 texels), and
  `preset::EXIT_TEAR` (1.0).

### Changed

- **An exit wound is torn, not just wide** (AG-149). `EXIT_SCALE`'s doc said an exit was "larger and more ragged than the hole it went in
  by"; the crater it was given was the entrance's round crater, 1.6 times wider, so one shot through a body left two circles. The preset's
  exit now calls `paint_world_torn` with `EXIT_TEAR`: a small hub with five to seven radiating splits, as a forensic exit is ("stellate or slit-like",
  Medscape, *Forensic Pathology of Firearm Wounds*; "bigger, more asymmetrical, and everted", the PMC review of gunshot wound patterns),
  and as Frances Glessner Lee's chest plates of about 1940 model it with a revolver's lead bullet (the entrance a small round hole, the exit a
  hub of six or seven bent splits). An entrance, a way back in after the first, and every other hit are round as they were. **Only `flay=` moves**
  in the recorders that have an exit (`capture_skinned --through` and `--guard`): no wet byte changes, so `digest=` holds, and every other recorder
  variant's digests equal 0.8.3's. The mask is built once per exit wound.

## [0.8.3] - 2026-11-01

### Changed

- **Blood dries dark and stays where it dried** (iteration 9, AG-148, `docs/research/dry-blood.md`). No recorder had ever looked at blood
  that had dried: they end seconds after the last shot and a stain takes 30 s. Looked at (`capture_skinned --dry 120`, below), the
  preset's blood was wrong twice. **(1) Dried spatter was erased.** The substrate's soak (`WetSettings::absorbency`, 0.15) took up to
  38 bytes from every texel over the drying, which is a spatter drop's whole film: after 1 800 ticks a cone of 1.5 texels' radius
  was gone, one of 2 kept 1 of 11 texels over the draw edge and one of 4 kept 19 of 47, so every drop a wound throws at a 512-texel
  wall (5.9 mm a texel) was gone or a pinhead once dry. A drying drop or pool is pinned to the surface and loses height, not area
  (Smith, Nicloux & Brutin 2020, `doi:10.1038/s41598-020-65465-4`: "the pool is pinned to the surface and the liquid phase evaporates
  at the surface. The volume loss corresponds then to a decrease in height"). `advance` now takes a wet texel's film down to
  `wetmap::STAIN_FLOOR` (33 of 255: the draw edge plus one cone step, tuned) and no lower, and takes nothing from a texel already
  under it; a pool's 255 still goes to 217. **(2) The flesh material never aged the colour.** `flesh.wesl` composited a film from its
  thickness alone, so a pool, a wound or a drop drawn with it stayed the colour it was born, only its gloss following the clock
  (measured on one clip, wet and dried: the floor pool `(119, 53, 59)` and `(124, 63, 68)`, the wall's drops `(142, 61, 62)` and
  `(148, 65, 63)`). The CPU's colour walk (`bloodstain::dry::appearance_with_fresh`, Bremmer et al. 2011,
  `doi:10.1371/journal.pone.0021845`) is now baked at its two stops into eight new rows of the flesh tables
  (`flesh::aged_blood_lut`, below the blood table, with a golden of its own; `the_flesh_tables_are_frozen` is untouched), and the
  shader interpolates between the stops by the wetness byte it already reads, in encoded sRGB, as the CPU does: fresh blood is the
  colour it always was to the bit, dried blood is dark brown and matte. The age is read at the texels, by the spline's own weights
  over the ones that have blood in them, so a fresh drop has no brown ring and a fresh drop beside an old one is visibly younger.
  **Measured**, the wall of `capture_skinned --through --floor 512` close up, 0.8.2 and now: 6 s after the shots 37 drops and
  1 884 px of blood against 40 and 2 478 (the soak no longer shrinks them while they dry); dried, 19 drops and 626 px
  `(146, 66, 64)` against 40 and 2 490 px `(77, 51, 46)`. `WetSettings::absorbency` is as it was; there is no setting that brings
  the erasure back. **Every recorder digest where blood soaks moves, on purpose**; `bench_carnage`'s does not (it never calls a
  canvas). **Cost**: `hit_tick_cost` retires 13.66 billion instructions before and 13.81 after (+1.1 %, the persistent blood
  keeps a few more texels in each tick); the shader reads the wet map 16 more times for a pixel with blood in it and 8 more
  table rows, and adds no texture, sampler or uniform. `flesh.wesl` is re-checked on WebGL2 (headless Chrome 143 over SwiftShader,
  `tools/webgl2/check.sh`): the flesh pipeline builds and draws blood with no error, and the same check fails on a shader with one
  type error added.

### Added

- `wetmap::STAIN_FLOOR`, `flesh::{AGED_ROWS, AGE_KNOT, aged_blood_lut, aged_lut_digest}`, and `capture_skinned --dry N` (blood
  dries in `N` ticks instead of 1 800, so a 360-frame clip shows the dried state: `--through --floor 512 --dry 120`).
- `tools/webgl2/`: the WebGL2 check `CLAUDE.md` asks for after a change to `flesh.wesl`, as a script (`check.sh <source-tree>`; a cold
  wasm build of Bevy takes about 38 minutes, an incremental one 30 s to a few minutes). `Cargo.toml` now also excludes
  `docs/media/*.gif` from the package, as `docs/*.gif` already were.

## [0.8.2] - 2026-10-31

### Changed

- **A drop is drawn at the size it is: round, crisp, and not diffused into a diamond** (iteration 8, AG-147,
  `docs/research/drop-edge.md`). On a default `GorePlugin` floor and wall the stains of a wound's spray read as rotated squares
  and rounded diamonds with a pale middle, all about one size, and iteration 6 and 7 put it down to the floor's texel and the
  flesh shader's edge. It was the canvas's own data. A speck (`paint_speck`) laid the stain's area as a blur twice its size, which a
  shader draws where it crosses 4 % of full (`WET_EDGE`, `DETAIL_EDGE`): two to four texels across for a drop of one, on a grid a
  rounded square, and a stamp or a speck then *spread*: the pass rounds `spread_rate / 4 · difference` to a byte from a difference of
  25, so a wet stain of five texels went from a centre of 255 to a plateau of 77 and the L1 ball of a four-neighbourhood, with every
  edge at 45°. The new `WetCanvas::paint_cone` writes the stain's own edge instead, after Green's distance fields ("Improved
  alpha-tested magnification...", Valve, SIGGRAPH 2007): the canvas under a drop is `DETAIL_EDGE − CONE_SLOPE · d` for `d`
  the signed distance from the edge, which the cubic B-spline the shader reads through reproduces exactly, so the 4 % line is the
  ellipse. Measured: within 1.5 % of the stain's area from a radius of 1.5 texels at any offset in the texel; round to a fifth of
  the radius; **with the substrate soaking nothing, 600 ticks leave every byte as painted** (its slope is under the 25 the spread
  rounds to a byte). `paint_drop` uses it for every droplet a floor, a wall or a skin takes under **eight** texels across
  (`FINE_TEXELS` was four, and a wall and a skin never took a speck: every droplet there was a spined stamp); a cloth keeps its
  blob. A drop is drawn no smaller than `CONE_MIN_SEMI` (0.7 texel), and no spine is drawn on a stain under eight texels, where it
  was under two texels wide. **The look**: a shot into a room leaves a fine spatter of round drops of the sizes the model
  throws, where it left a field of rounded squares two to three times too big. **Cost**, `hit_tick_cost`, median of six runs
  alternated with 0.8.1: a heavy wound facing a wall 18.8 to 9.3 ms on its hit tick and 15.5 to 12.1 on every tick after, a wound
  facing a wall 4.9 to 2.7 and 5.7 to 3.7, a bullet on a floor 3.0 to 2.7; no case slower; the whole run retires 25 % fewer
  instructions (18.0 to 13.6 billion). **Every digest that has a drop under eight texels moves, on purpose**: `capture_skinned`'s
  default, `--break`, `--shirt`, `--through`, `--guard` and `--blows`, and `capture_preset`; `--stroll` and `--quiet`, `bench_carnage`
  (`0x4366a8b64352c85a`) and every golden are 0.8.1's. The drying still takes a thin film: a cone of a texel's radius soaks away
  by the time it is dry, as faint spatter always has (`absorbency`), and a larger one shrinks from the rim. A cone is not a
  mass: its bytes do not sum to the stain's area.

### Added

- `WetCanvas::paint_cone(uv, semi, direction, tick)` and `wetmap::{CONE_SLOPE, CONE_MIN_SEMI, CONE_MAX_SEMI}`: see Changed. Six
  contract tests in `tests/wetmap.rs` (`a_cone_is_drawn_at_the_size_it_was_given_and_round`, `a_cone_is_an_ellipse_along_its_direction`,
  `a_cone_does_not_creep_while_it_dries`, and the reproducibility, least-size and refusal ones). `paint_speck` and its
  tests are kept, for a caller who wants a stain's blood as a mass; the preset no longer calls it.

### Fixed

- The test helper `preset::tests::stains_on` counted a stain at 32 of 255, a level only a drop's blood *diffused over a plateau several
  times its own area* reached; a drop of its own size has an edge at the draw edge (`DETAIL_EDGE`, 10 of 255) and a middle a little
  above it, and was missed. It counts at the draw edge now; what `a_shot_leaves_a_fan_of_drops_on_the_floor_not_a_few_blobs` asks of
  the count (at least 15 stains, four times the mist-free count, none wider than 12 cm) is unchanged. Assumed decision (owner may
  override): the assertion's *thresholds* stand; its *measure* followed the model it measured.

## [0.8.1] - 2026-10-31

### Changed

- **A wounded scene costs the caller about two fifths less a tick, with every byte unchanged** (iteration 7, AG-146,
  `docs/research/wet-tick-cost.md`). A heavy wound facing a wall took 28 ms of one tick, and the blood it left kept the scene at
  26 ms a tick for as long as it was wet: the canvases' own passes were the larger share of every hit, and the stamp the rest.
  Five reductions, each exact. `bloodstain::stain::rasterise` tabulates, per angular bucket, which spines can reach it and an
  upper bound on the rim there, so a texel outside the stain skips its spines and the rest walk only the spines that can reach
  them, in the same order (`rasterise_writes_the_mask_its_definition_does`, widened to the ceiling of 24 spines and 24
  satellites and to the impacts `stain_shape` makes, holds every byte to the mask by definition). `WetCanvas`'s shade pass keeps a
  table on `(level, age)` where one remembered pair thrashed along every rim; its spread reads the flux from a table of the 511
  differences and its soak is tabulated by age; both skip the bare texels whose surroundings are bare, which is most of what a
  tick covers; and **a tick now covers a spray's own blood and not the tiles it fell in**: the live tiles kept each tile's
  bounding rectangle of blood, which for a spray of fine drops was 9 388 texels covered for 1 144 with blood
  (`a_spray_is_covered_by_its_own_blood_and_not_by_its_tiles` fails on 0.8.0), and the passes are paid per texel covered. The
  passes are held to their definitions on every tick (`the_passes_are_what_their_definitions_say`: `wet` and every pixel of both
  images, four scripted runs). Median hit tick of 45 hits one tick apart, alternated with 0.8.0 eight times on a loaded host:
  a bullet on a floor 4.5 to 3.0 ms, facing a wall 6.9 to 4.8, a heavy wound 8.3 to 5.0, a heavy wound facing a wall 28.5 to
  18.7; and the tick after the last hit, with nothing new thrown, 4.8 to 2.8, 9.4 to 5.7, 5.6 to 2.8 and 26.5 to 15.4. The whole
  of `hit_tick_cost` retires 41 % fewer instructions (30.5 to 18.0 billion). **No digest moves**: `bench_carnage`
  `0x4366a8b64352c85a`, every recorder variant's `digest=`, `flay=` and `dermis=`, and every one of the 6 480 frames the
  recorders write are 0.8.0's, byte for byte. A canvas holds up to 8 KiB more once it has shaded blood and a byte an age of its
  drying span (`WetCanvas::heap_bytes` counts both).

### Added

- `preset::tests::hit_tick_cost` also prints `wet_tick_ms_median_*`, the median of the 90 ticks after the last hit with nothing
  new thrown: every frame of a scene while its blood is wet, which until now nothing measured.

## [0.8.0] - 2026-10-31

### Changed

- **A floor takes what a wound throws as drops, not as blobs** (iteration 6, `docs/research/floor-spatter.md`). A
  default `GorePlugin` floor under a bullet-sized shot took about nine ragged blobs of ten centimetres, each the
  *placement* radius of a droplet (2 to 12 cm) stamped as a sheet's stain: at 12 mm a texel, a few pieces of jam.
  Now each droplet a floor takes is its own stain, as a wall's and a skin's have been since 0.5.62: `hard_stain`'s size
  and slant from the speed it lands at, found on the floor by a cast through the mesh's index instead of the canvas
  walking every triangle once per droplet, so a hit on a floor of twenty thousand triangles costs 4 ms where it cost 9.
  A stain under four texels across, which a canvas cannot give spines, is a `WetCanvas::paint_speck`. **A wound also
  throws `GoreDials::floor_mist` fine droplets at the floors for each droplet it throws at everything** (`9`, so a
  crater's 9 droplets are 90): the same stream, cast at the same surfaces and caught by the first they reach (a wall
  across the room keeps the floor behind it dry), stamped only on a floor. A shot into a body in a three-metre room
  left 4 stains on the floor (the new stains, no mist) and now leaves 25 (`a_shot_leaves_a_fan_of_drops_on_the_floor_not_a_few_blobs`);
  **a wall, a skin and a cloth take exactly what they took**, pixel for pixel in `capture_skinned --through`. Every
  recorder digest whose clip has a hit and a floor moves on purpose (`capture_preset`, and `capture_skinned`'s default,
  `--through`, `--shirt`, `--break`, `--blows`), and `--stroll`, `--quiet` and `--guard` keep theirs.
  **Breaking, pre-1.0 so the next release is a MINOR**: `GoreDials` has a new public field, `floor_mist`; a literal
  that names every field adds `..default()`. A caller who wants the old throw states `floor_mist: 0.0`.

- The README follows the dark-incorporated org's documentation guide (250 lines at most; it was 646): the org banner, badges, a Status and a Why line, an 18-line quick start compiled as a doctest, a still image, and "What you can do" bullets. Every section it carried moved unchanged into `docs/guide/` and `docs/gallery.md`, and the guide's full quick start is still compiled as doctests. Image and file links are absolute URLs, so docs.rs shows them.

### Added

- `WetCanvas::paint_speck(uv, semi, direction, tick)` and `wetmap::{SPECK_MAX_SEMI, SPECK_GROW, SPECK_FLOOR}`: a droplet's
  stain too small for a stamp, the blood of an ellipse of semi-axes in texels laid over a soft round footprint, placed to a
  fraction of a texel, so the bytes laid sum to its area (`a_speck_lays_its_area_whatever_its_size_and_where_it_sits`) and it
  is round about where it is centred (`a_speck_is_symmetric_about_where_it_is_centred`). A stamp of two texels peaked at 73
  of 255, one of three at 243 and a slanted one of one at 18 (measured on the floor at 12 mm a texel), and a stain smaller
  than the stamp's floor was drawn as a whole texel; a speck of an eighth of a texel adds an eighth of full coverage. A
  hard-edged disc on a grid is a square after the cubic B-spline a shader reads the canvas through, so the area is spread
  over a bump twice the ellipse's size (`SPECK_GROW`, tuned). Only multiplies, adds and compares on `f32`, no transcendental: the bytes are the same everywhere.
- `GoreDials::floor_mist` and `preset::GORE_FLOOR_MIST` (`9`): see Changed.
- `capture_skinned --floor N` dresses the floor at `N` texels a side instead of the dial's 256, to see spatter at 6 mm a
  texel (`--floor 512`). An ignored measurement test, `preset::tests::hit_tick_cost`, prints what one hit costs the tick it
  lands on in a room with a floor of two triangles and of twenty thousand and a wall (`cargo test --release --lib --
  hit_tick_cost --ignored --nocapture`).
- `DermisCanvas::digest()` (#8): FNV-1a over the bruise-and-burn canvas's bytes, the form `ClothHoles::digest`
  already had (which now delegates to it). `capture_preset` and `capture_skinned` append `flay=<hex>` (every
  `FlayCanvas::digest`, the craters) and `dermis=<hex>` (every `DermisCanvas::digest`, the bruises and burns) to
  their digest line, each a sorted fold over the world **sampled at every frame of the clip**: at the end of
  `capture_preset` and of `capture_skinned --break` the body has broken and no skin canvas is left, and a
  fold of the world as it then stands read `cbf29ce484222325`, the empty fold, whatever the craters had been.
  **No `digest=` value moves and no frame changes**: until now neither a crater's depth nor a bruise's place
  nor a burn's zones was in the reproducibility line, so `capture_skinned --blows` printed the default's line
  and a pass could have made any of them run-dependent unseen. Research: `docs/research/digest-fold.md`.

## [0.7.0] - 2026-10-31

### Changed

- `GoreDials::default()` throws at game scale: its `blood.spatter_speed_scale` is `0.25`
  (`preset::GORE_SPATTER_SPEED_SCALE`), where it inherited `BloodSettings::default()`'s `1.0`. The paper's
  8 to 40 m/s launch speeds, landed by this crate's drag-free parabola, put a droplet from a chest-height
  wound 1.5 to 55 m out (median 4.8 m, 21 of 48 within 4 m), so a three-metre floor under a default
  `GorePlugin` took almost none of what a wound threw, and `preset::tests::a_wound_stains_the_floor_beneath_it`
  was red from `0.5.64` to `0.7.0`. At `0.25` the same wound lands 0.6 to 5.1 m out (median 1.5 m, 47 of 48
  within 4 m), the 0.25 to 4 m field Comiskey, Yarin & Attinger measured their stains over.
  **The scale now compresses the flight to a floor and nothing else**: a droplet a wall, a cloth or a body
  part catches is cast and sized at the paper's launch speed whatever the dial says. Scaled too, a wall's
  stipple came out two thirds the size and nearly one size (`capture_skinned --through`). So a default
  `GorePlugin` stains its walls exactly as it did and its floors where the spatter was measured; a caller who
  authored the dial (`0.25`, or `gore_tier`'s `0.12`) keeps its floors and gets walls, cloth and skin stained at
  the paper's speed. **This moves every recorder digest whose clip has a hit and a floor under the default
  dials** (`capture_preset`, and `capture_skinned` except `--stroll` and `--quiet`); `BloodSettings::default()`
  is unchanged and so is every bloodstain golden. One that wants the old floor throw sets
  `spatter_speed_scale: 1.0`. Research: `docs/research/spray-reach.md`.

### Fixed

- `fracture_skeleton` refuses what it cannot cut exactly, by name (#7). A part that is not a
  `TriangleList` (a `TriangleStrip` or a `LineList`, which Bevy's glTF loader produces for
  `Mode::TriangleStrip` and `Mode::Lines`) or whose index count is not a multiple of three is
  `SkeletalRefusal::NotTriangleList { part }`, where it was cut as a list with the remainder dropped without
  a word; an index past the vertex buffer is `BadIndex { part, triangle }`, where it was reported as
  `SplitTriangle` ("the body is not rigid"); and a vertex with weight on a joint outside its dominant joint's
  group, or on a joint no group holds, is `SplitVertex { part, vertex }`, where the module's own "a
  soft-skinned body is refused" was not kept and the piece jumped at the break. A group's own joints may
  still share a vertex. Three new `SkeletalRefusal` variants: `fracture_skeleton` is in no published
  release, so nothing a consumer holds breaks. `read_part` reads the indices with `try_indices_option`
  rather than the accessor that `expect`s on a mesh extracted to the render world. Research:
  `docs/research/skeleton.md`.
- `tools/qa.sh` finishes a round on a default build (#5, `AG-140`). The six `vfx` examples (`carnage`,
  `capture_carnage`, `pooling`, `capture_pooling`, `ribbons`, `capture_ribbons`) stay on its roster but
  run only when `QA_FEATURES` names `vfx`, which also adds it to the build step; otherwise each gets one
  `missing.txt` line naming the feature and the round goes on, where it used to end at the first of them
  with "no built binary". A binary the default build should have made and did not still ends the round.
  A `vfx` binary an earlier build left behind is not run in a round that did not build `vfx`.
- `tools/qa.sh` renders the skinned body: `capture_skinned` is a recorder name, and so is each variant
  `docs/DEMOS.md`'s recipe records (`capture_skinned_break`, `_shirt`, `_through`, `_guard`, `_spray`,
  `_stroll`, `_blows`, `_quiet`), with the same flags; the autopiloted `skinned` example is on the
  windowed list with a key list taken from its own key reads.
- `tools/qa.sh` gives each windowed run a watchdog budget that grows with its script, `max(180, 60 +
  frames/4)` s, where it was a flat 180 s that reported a slow, busy host as a missing example
  (`skinned` and `sever` hit it). `QA_TIMEOUT` is still one flat limit for every run.
- `README.md` and `docs/DEMOS.md` say what `Cargo.toml` says (#6). The README is the crate-root rustdoc, so
  docs.rs and crates.io showed it: "Requires Bevy 0.19" over `bevy = "0.20.0"`, an install line of
  `bevy_carnage = "0.5"` over `version = "0.7.0"`, `vfx` ticked as a default feature when `default` is
  `["serde", "flesh"]` (and `vfx` cannot be enabled on Bevy 0.20 at all until `bevy_hanabi` publishes for
  it), a banner and a Contributing section calling this repository a read-only mirror of
  `foundation_vs_slop`, and a Status paragraph naming `0.5.64`. All now state the manifest's own values
  and the arrangement `CLAUDE.md` records (this repository is the source of truth; the monorepo's copy is
  frozen at `0.5.64` on Bevy 0.19). `docs/DEMOS.md` no longer says both features are on by default. The
  README gains a `0.6.0` / `0.7.0` section and puts `fracture_skeleton` and `SkeletalRefusal` under
  "Skinned meshes". `Cargo.toml`'s comment about the excluded GIFs no longer says they are served from a
  mirror. Research: `docs/research/readme-truth.md`.

### Added

- `tests/qa_roster.rs`: runs `tools/qa.sh` dry and for real against an empty binary directory, and
  holds what it plans and reports to `Cargo.toml` and `examples/`.
- `tests/readme.rs`: four tests that read `README.md` and `Cargo.toml` and fail when they disagree on the
  Bevy version (`Requires **Bevy X.Y**` against `[dependencies] bevy`), the install line's `bevy_carnage =
  "A.B"` against `[package] version`, the features the table ticks against `default = [..]`, and the version
  the Status paragraph says is in the repository against `[package] version`. A release that moves one of
  those numbers fails `cargo test` until the README moves with it.
