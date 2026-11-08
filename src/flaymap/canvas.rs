//! **The canvas: one `Vec<u16>` of removed tissue, in row-major order, and three passes over it.**
//!
//! Everything a flaymap decides lives in this file, on the CPU, in integers. The three `Image`s exist
//! so a renderer has something to sample; nothing in this crate ever reads one back.

use bevy::asset::{Assets, Handle, RenderAssetUsages};
use bevy::image::Image;
use bevy::log::warn;
use bevy::math::{Vec2, Vec3};
use bevy::mesh::Mesh;
use bevy::prelude::Component;
use bevy::prelude::{Reflect, ReflectComponent};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::transform::components::GlobalTransform;
use crate::cross_section::{Layer, Layers, Region, texel_at, texel_mean, value_noise, worley};

use crate::flaymap::digest::Fnv1a;
use crate::flaymap::settings::FlaySettings;
use crate::flaymap::uv::{Pick, mesh_key, ray_uv};

/// **How much world one UV unit is taken to be**, metres.
///
/// A `Mesh`'s UV parameterisation carries no scale — nothing in a texture coordinate says whether the
/// unit square covers a face or a whole body — so the bridge from a texel index to the millimetre
/// position [`crate::cross_section::texel_at`] wants has to be *stated somewhere*. It is stated here,
/// once, as a constant: **one UV unit is one metre**, the same constant and the same argument
/// `wetmap` makes for a stain's footprint.
///
/// A caller whose atlas covers a two-metre actor therefore has a canvas at half the texel density this
/// constant implies, and either accepts the resolution or authors a bigger canvas. The alternative — a
/// per-canvas scale parameter — would be a dial every call site had to agree on to get comparable
/// tissue grain, and disagreement would be invisible until two actors' flayed skin looked like two
/// different animals.
pub const UV_SPAN_M: f32 = 1.0;

/// Meshes remembered for the once-per-mesh warning before the memo stops growing.
///
/// A cap rather than an unbounded set: the memo exists to stop log spam, and an actor is built from a
/// handful of meshes. Past the cap the oldest key is dropped, so a long session that keeps meeting new
/// broken meshes may warn about one twice — the right way round for a memo whose other failure mode
/// would be unbounded memory in a shipped game.
const WARN_MEMO: usize = 16;

/// Hundredths of a millimetre per millimetre — the unit the depth buffer counts in.
///
/// A `u16` of hundredths reaches 655.35 mm, two orders of magnitude past the deepest region
/// [`Layers`] describes, and 10 µm is a tenth of a Haversian canal. Integers rather than an `f32`
/// because the buffer is the thing the digest folds: a float accumulation would make the wound depend
/// on the order the hits were summed in.
const HMM_PER_MM: f32 = 100.0;

/// The tissue grain a texel has to resolve before it may show it, millimetres: fat lobules are two
/// across and the perimysium net about three. See [`FlayCanvas::shade`].
const GRAIN_MM: f32 = 2.0;

/// Depth resolution of the grain-averaged tissue table, hundredths of a millimetre: 0.1 mm, a third
/// of the thinnest thing the tissue model draws, the epidermis over the first 15 % of the skin.
const TISSUE_BIN_HMM: u16 = 10;

/// How far the colour lookup's depth is torn either way, at most, millimetres. See
/// [`FlayCanvas::shade`]. Enough to rag the rings and the rim without blurring the order of the
/// bands on the thick rows; on the head's 2 mm bands an edge can skip one.
const RAGGED_MM: f32 = 3.0;

/// **How far a fully torn crater's longest split reaches past the rim**, in crater radii: `1.3`, so a split of a crater of
/// radius `r` runs out to `2.45 r` from the centre at most. See [`FlayCanvas::paint_uv_torn`]. **Tuned** against the exit wound
/// of a revolver's lead bullet in Glessner Lee's chest plate, whose longest split runs about nine times the hub's radius
/// (`docs/research/torn-exits.md`); a crater here is the *wound's* radius, the hub and the margin the skin burst
/// outward in, so the shorter reach.
pub const TEAR_REACH: f32 = 1.3;

/// **How deep a split is, as a share of the crater's depth**: `0.5`. A split opens the skin and shows what is under
/// it; it does not bore into the muscle as the crater at its hub does. **Tuned**, a design default.
pub const TEAR_SPLIT_DEPTH: f32 = 0.5;

/// **How wide a split is at its root, in texels**: `2.0`, tapering to a point at its tip. **Tuned.**
pub const TEAR_SPLIT_WIDTH: f32 = 2.0;

/// The coarse octave's cell, millimetres — a few texels on a 128 canvas, so the tears are shapes
/// rather than speckle.
const RAGGED_CELL_MM: f32 = 24.0;

/// **How much of a crater's radius its wall takes**: `0.45`. The crater's depth rises from nothing at the rim to its floor over the outer 45 % of the radius, not over all of it (`1.0` is the
/// bowl of 0.8.7, whose rim had zero slope and so spent a third of the radius on skin and fat). A wound has an edge: "Entry wounds are typically smaller, with well-defined margins" (PMC12791185,
/// `docs/research/wound-margin.md`). **Tuned** by eye and by the ring's share of the radius.
const WALL: f32 = 0.45;

/// **How much of a crater's depth keeps deepening to its centre** instead of ending in the floor the wall made: `0.30`, so the floor is a pit and not a plateau (a plateau stacks, hit on hit, into a
/// bore). `1.0` is the bowl. **Tuned.**
const PIT: f32 = 0.30;

/// **How out of round a crater's rim may be**, as a share of its radius, per harmonic: `0.035` for each of three harmonics (3rd, 5th and 8th of the angle), at full effect from `RIM_FULL_TEXELS` of
/// radius. A big wound is ragged; a bullet's is round (`docs/research/torn-exits.md`), so it fades in from `RIM_FROM_TEXELS`. **Tuned.**
const RIM_RAG: f32 = 0.035;
/// Radius, in texels, under which a crater's rim is round: `8`.
const RIM_FROM_TEXELS: f32 = 8.0;
/// Radius, in texels, from which a crater's rim is as ragged as it gets: `32`.
const RIM_FULL_TEXELS: f32 = 32.0;

/// **The size of a patch of pale nodules**, millimetres: `40`, a slow value noise that scales how many nodules in a place stand up and how pale they are. **Tuned.**
const LUMP_PATCH_MM: f32 = 40.0;
/// **The floor's lump cell**, millimetres: `16`, a nodule of fat or torn fascia the size of a grape (the references' nodules are 4 to 16 mm, 5 to 8 % of the wound's width: W2, W4), four texels on the
/// 256 canvas. A second lattice 2.3 times coarser stands up as the big nodules among the small, a slow swell (55 mm) lies under both, and the lattice is warped by a slow noise so the cells are
/// not a grid (a diagonal weave appeared at 12 mm cells with a second octave at half the cell; one octave at 16 mm removed it). Lumps are at full strength while a texel is under half a cell and gone from
/// 0.85 of one (8 mm and 13.6 mm: the 256 canvas is 3.9 mm a texel, a 128 one 7.8). **Tuned** against the reference set.
const LUMP_CELL_MM: f32 = 16.0;
/// **How deep a texel must be peeled before it is lumpy**, millimetres: `4`. The skin's lip stays smooth. Lumps stop again where the cortex starts (bone is smooth). **Tuned.**
const LUMP_FADE_MM: f32 = 4.0;
/// **How far a lump moves the depth the tissue is looked up at**, millimetres: `3.5`. A nodule is shallower, a crevice deeper, so fat and muscle show in patches. **Tuned.**
const LUMP_LOOK_MM: f32 = 3.5;
/// **How high a lump stands in the normal image**, millimetres: `5.0`: the lump's own height, added to the depth's slope, so the shader lights it. **Tuned**: under the blood film that covers a fresh
/// wound the albedo of a nodule barely shows, and the lighting of its relief is what the eye reads.
const LUMP_RELIEF_MM: f32 = 5.0;
/// **How much darker a crevice is**: `0.6` of the colour at the seam of a cell. **Tuned.**
const LUMP_DARK: f32 = 0.6;
/// **How much brighter the top of a nodule is**: `1.1` at the cell's centre. **Tuned** (the references' 95th percentile of luminance is 134 to 165 where a smooth crater's is 110).
const LUMP_LIGHT: f32 = 1.1;
/// **How much paler a nodule is**, toward `LUMP_NODULE`: `0.3` at the cell's centre. **Tuned**: more of it greyed the wound's red (the references' red minus green is 86 to 109).
const LUMP_PALE: f32 = 0.3;
/// **How deep the skin's lip reaches**, millimetres: `3`. A peeled texel shallower than this is the cut edge of the skin, pale and raised, fading to the wound's wall by this depth. The 0.8.7 wall
/// spent a third of the radius on it; with [`WALL`] it is a ring a texel or two wide. **Tuned.**
const LIP_MM: f32 = 3.0;
/// **How much of the way to [`LIP_COLOUR`] the rim texel goes**: `0.75`. **Tuned** against W1, W2, W3, W5, whose lip is paler than the skin round it.
const LIP_PALE: f32 = 0.75;
/// The cut edge of the skin, paler and pinker than the skin round it (encoded sRGB `(0.93, 0.80, 0.72)`; the intact skin of the recorders is about `(0.78, 0.66, 0.60)`). **Tuned.**
const LIP_COLOUR: [f32; 3] = [0.93, 0.80, 0.72];
/// **How high the lip stands**, millimetres: `1.8`, in the normal image: a rolled edge, not a cliff. **Tuned.**
const LIP_RELIEF_MM: f32 = 1.8;
/// The torn fascia and fat a nodule shows, encoded sRGB: `(0.80, 0.52, 0.44)`. **Tuned** against W2 and W4.
const LUMP_NODULE: [f32; 3] = [0.80, 0.52, 0.44];
/// **How much wetter a nodule is** (roughness times `1 - 0.6 k`) and how much rougher a crevice (`+ 0.25`). **Tuned.**
const LUMP_WET: f32 = 0.6;

/// **What a paint call found, and the one-shot handoff to whatever owns the skeleton.**
///
/// Returned by [`FlayCanvas::paint_uv`] and [`FlayCanvas::paint_world`] rather than pushed as a
/// message, because the crate that peels the skin does not know which of a caller's systems owns the
/// bone underneath. [`crate::flaymap::BoneExposed`] is the message type for the caller to *forward* it on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Handoff {
    /// **The deepest tissue this stamp is now standing on** — the layer at the maximum depth over the
    /// texels this call covered, after the call.
    ///
    /// Per call, not per canvas, and that is the useful reading: it answers "what did *this* hit
    /// reach", which is what picks an impact sound, a particle or a wound decal. A call that painted
    /// nothing — a refused UV — reports [`Layer::Skin`], because nothing was reached.
    pub deepest_layer: Layer,
    /// **True on exactly one call per canvas**: the first one in which any texel's depth crosses
    /// `Layers::starts_mm()[3]`, the top of the cortex.
    ///
    /// Bone is exposed once. A flag that stayed true would make every later hit re-announce it, and a
    /// consumer that spawns a fracture proxy or a bone-scrape sound on the announcement would spawn
    /// one per shot for the rest of the fight.
    pub bone_reached: bool,
    /// Where the bone came through, in this canvas's UVs — the stamp centre of the call that did it.
    /// `Some` on exactly the call where [`bone_reached`](Self::bone_reached) is true.
    pub first_bone_uv: Option<Vec2>,
    /// **Where the hit landed in the mesh's own space**, so a consumer can fracture, spawn or attach
    /// at the wound without searching the mesh for the texel that a UV came from.
    ///
    /// `Some` from [`paint_world`](FlayCanvas::paint_world), which found it by ray; `None` from
    /// [`paint_uv`](FlayCanvas::paint_uv), which was handed a texture coordinate and cannot invert
    /// one — a UV names a point on an atlas, and an atlas seam maps one UV to several places on a
    /// body. Guessing there would be worse than saying nothing.
    pub at: Option<Vec3>,
    /// The hit triangle's **geometric** normal, mesh-local and normalised: `e1 × e2` from the
    /// winding, not the interpolated shading normal, because what a caller aligns a fracture plane or
    /// a decal to is the plane the surface lies in.
    ///
    /// `Some` from [`paint_world`](FlayCanvas::paint_world) and `None` from
    /// [`paint_uv`](FlayCanvas::paint_uv), for the same reason [`at`](Self::at) is. Zero for a
    /// degenerate triangle, and **not** flipped toward the ray: the intersection is two-sided, so
    /// which side was struck is the caller's dot product to take.
    pub normal: Option<Vec3>,
}

/// **Texture-space flaying for one actor.** Three images out, and the authority stays here.
///
/// # The state is one buffer, and its order is the canonical order
///
/// `depth: Vec<u16>` — hundredths of a millimetre of tissue **removed** at that texel — in
/// **row-major** order. That order *is* the order [`shade`](Self::shade), [`digest`](Self::digest) and
/// [`exposed_area`](Self::exposed_area) walk, so nothing here needs a sort and nothing here may add
/// one: a sort would be a second answer to a question the layout already answers, and the digest would
/// then depend on which answer ran.
///
/// Depth is **monotone**. Tissue does not grow back, so every pass in this file either adds to a texel
/// or leaves it alone, and the buffer saturates at `Layers::span_mm()`. That is what makes the layer
/// sequence — skin, fat, muscle, cortex, marrow — a one-way walk rather than a state machine.
///
/// # What the caller owns
///
/// The hits and the shading. [`paint_uv`](Self::paint_uv) takes the tick number because this crate has
/// no clock and will not guess one; [`shade`](Self::shade) is called by the caller after the last paint
/// of a tick, because only the caller knows when that was. The plugin registers only the upload
/// budget, which is the one part with no gameplay opinion in it.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct FlayCanvas {
    /// Edge length in texels. Always ≥ 1.
    size: u32,
    /// Which thickness row this body part uses. Carried so a caller can ask a canvas what it is
    /// rather than keeping a parallel map of entity → region.
    region: Region,
    /// The depths this canvas peels through, outside to inside.
    layers: Layers,
    /// Hundredths of a millimetre removed per texel, row-major. **The authority.**
    depth: Vec<u16>,
    /// The current stamp's radial falloff, `0..=255` per texel, row-major over a `px × px` square.
    /// Scratch, kept so a stamp does not allocate; rebuilt per call, because a stamp that reused a
    /// mask built for another radius would peel the wrong footprint.
    mask: Vec<u8>,
    albedo_handle: Handle<Image>,
    rough_handle: Handle<Image>,
    normal_handle: Handle<Image>,
    /// RGBA8 sRGB bytes: base colour where nothing is peeled, tissue where something is.
    albedo_px: Vec<u8>,
    /// RGBA8 **linear** bytes: G is roughness, B is metallic. See [`roughness`](Self::roughness).
    rough_px: Vec<u8>,
    /// RGBA8 **linear** bytes: the tangent-space normal of the peeled surface. See
    /// [`normal`](Self::normal).
    normal_px: Vec<u8>,
    /// The grain-averaged tissue per 0.1 mm of depth, filled on first use, and the
    /// `(tile_mm bits, seed)` it was filled for. See [`shade`](Self::shade).
    tissue: Vec<Option<([u8; 3], u8)>>,
    tissue_key: (u32, u32),
    /// The height each texel's lump stands at, twentieths of a millimetre, signed: what [`shade`](Self::shade) added to the depth's slope there, kept because the normal of a texel on the edge of a
    /// redrawn rectangle reads its neighbours' lumps from an earlier shade. Zero where nothing is peeled. **Not in the digest**: a pure function of the depth buffer and the settings.
    lump: Vec<i8>,
    /// The intact surface, as bytes, so a texel at depth 0 does not re-encode it per shade.
    base_rgba: [u8; 4],
    /// The intact surface's roughness byte.
    base_rough: u8,
    /// Earliest tick at which the CPU state diverged from what was last uploaded, or `None` when the
    /// two agree. **Oldest-dirty-first** ordering in the plugin's budget reads this.
    dirty_since: Option<u32>,
    /// The tick of the most recent paint that changed anything.
    ///
    /// Only [`shade`](Self::shade) reads it, and only to answer a question it has no tick of its own
    /// for: if a settings change repaints the pixels of a canvas that is already uploaded, the
    /// divergence really did originate with that paint, so that is the tick the upload queue is
    /// ordered by.
    painted_at: u32,
    /// Texels whose depth has reached the cortex — bone, in one integer.
    bone_texels: u32,
    /// Whether the one-shot bone handoff has already fired. See [`Handoff::bone_reached`].
    bone_handed_off: bool,
    /// Meshes already refused, so a UV-less mesh warns exactly once instead of once per hit.
    warned: Vec<u64>,
    /// Whether a paint has already been refused for unusable input, so a caller feeding a NaN in a
    /// loop gets one line rather than one per frame.
    warned_input: bool,
    /// **The texels whose depth has changed since the last [`shade`](Self::shade)**, as the rectangle
    /// round them, `[x0, y0, x1, y1]` inclusive, or `None`. A shade redraws these and, for the
    /// normals, which read their neighbours, one texel round them; everything else already shows the
    /// depth it holds. Until 0.5.59 every shade redrew every texel, and the preset shades after each
    /// hit, so a shotgun's twelve pellets redrew a whole canvas twelve times: 2.3 ms of a 3.5 ms tick.
    stale: Option<[i64; 4]>,
    /// The texel size, `tile_mm` and `seed` the pixels were last shaded with, as bits, or `None` before
    /// the first shade: a shade with other settings redraws every texel, since the palette moved.
    shaded_for: Option<(u32, u32, u32)>,
}

impl FlayCanvas {
    /// **A blank canvas and its two images.**
    ///
    /// `size` is the edge length in texels; **128 is the practical default and 256 the ceiling.** The
    /// arithmetic is the reason, not taste: a 128×128 canvas at `Rgba8UnormSrgb` is
    /// `128 · 128 · 4 = 65 536` bytes — 64 KB per upload — and this crate owns **two** images per
    /// actor, so one canvas costs 128 KB of `Assets<Image>` writes every time it is flushed. At the
    /// shipped budget of four canvases per frame that is 512 KB/frame; at 512 it is 8 MB/frame, which
    /// is a bandwidth budget rather than a texture.
    ///
    /// `layers` is passed in rather than derived from `region` so a caller with its own thickness
    /// table — the cortex row in particular is `cross_section`'s own number rather than a
    /// measured one — uses it here as well as in the cross-section bake. Pass
    /// `Layers::for_region(region)` for the measured rows.
    ///
    /// `base_srgb` and `base_roughness` are the **intact surface**: unbroken skin, or whatever the
    /// actor wears. They are written into the canvas on the CPU, which is why this crate ships no
    /// shader and no asset — there is nothing left to blend in WGSL.
    ///
    /// A `size` of 0 is corrected to 1 and warns — a zero-extent texture is not a canvas.
    pub fn new(
        images: &mut Assets<Image>,
        size: u32,
        region: Region,
        layers: Layers,
        base_srgb: [f32; 3],
        base_roughness: f32,
    ) -> Self {
        let size = if size == 0 {
            warn!("flaymap: a canvas of size 0 has no texels; using 1");
            1
        } else {
            size
        };
        let base_rgba = [enc(base_srgb[0]), enc(base_srgb[1]), enc(base_srgb[2]), 255];
        let base_rough = enc(base_roughness);
        // **The four channels of the metallic-roughness image, and only two of them are Bevy's.**
        // G carries roughness and B carries metallic — tissue is a dielectric, so B stays 0 — and R
        // and A are free for the depth buffer's own two numbers: R is how deep this texel is peeled
        // as a fraction of the layer table's span, A marks a peeled texel. Intact skin is 0 in both.
        let base_rough_rgba = [0, base_rough, 0, 0];
        // A flat tangent-space normal, `(0, 0, 1)` encoded.
        let flat_normal = [128, 128, 255, 255];

        let extent = Extent3d { width: size, height: size, depth_or_array_layers: 1 };
        // `MAIN_WORLD | RENDER_WORLD` because `flush` rewrites the pixels every time they change: with
        // `RENDER_WORLD` alone the main-world copy is dropped after the first upload and every
        // subsequent write would land in nothing.
        let usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
        let albedo = Image::new_fill(
            extent,
            TextureDimension::D2,
            &base_rgba,
            // sRGB, because this image is sampled as colour by `StandardMaterial::base_color_texture`.
            TextureFormat::Rgba8UnormSrgb,
            usage,
        );
        // **Linear, not sRGB.** Roughness and metallic are material data, not colour; glTF's own
        // metallic-roughness textures are linear for the same reason, and sampling this one through an
        // sRGB decode would bend the gloss curve.
        let rough = Image::new_fill(
            extent,
            TextureDimension::D2,
            &base_rough_rgba,
            TextureFormat::Rgba8Unorm,
            usage,
        );
        // Linear for the same reason: a normal is a direction, not a colour.
        let normal = Image::new_fill(extent, TextureDimension::D2, &flat_normal, TextureFormat::Rgba8Unorm, usage);

        let texels = (size as usize) * (size as usize);
        Self {
            size,
            region,
            layers,
            depth: vec![0; texels],
            mask: Vec::new(),
            albedo_handle: images.add(albedo),
            rough_handle: images.add(rough),
            normal_handle: images.add(normal),
            albedo_px: base_rgba.iter().copied().cycle().take(texels * 4).collect(),
            rough_px: base_rough_rgba.iter().copied().cycle().take(texels * 4).collect(),
            normal_px: flat_normal.iter().copied().cycle().take(texels * 4).collect(),
            tissue: Vec::new(),
            tissue_key: (0, 0),
            lump: vec![0; texels],
            base_rgba,
            base_rough,
            dirty_since: None,
            painted_at: 0,
            bone_texels: 0,
            bone_handed_off: false,
            warned: Vec::new(),
            warned_input: false,
            stale: None,
            shaded_for: None,
        }
    }

    /// The base-colour image. Feed it to `StandardMaterial::base_color_texture`
    /// (`bevy_pbr-0.19.0/src/pbr_material.rs:57`).
    pub fn albedo(&self) -> Handle<Image> {
        self.albedo_handle.clone()
    }

    /// The metallic-roughness image. Feed it to `StandardMaterial::metallic_roughness_texture`
    /// (`bevy_pbr-0.19.0/src/pbr_material.rs:170`).
    ///
    /// **Roughness is the green channel and metallic is the blue one** — stated in that file at
    /// `:153-154`: *"The blue channel contains metallic values, and the green channel contains the
    /// roughness values."* B stays 0, so tissue stays a dielectric.
    ///
    /// **R and A are the flaymap's own data channels**, because Bevy reads only G and B from this
    /// image and a second texture per actor would be a second upload. Per texel:
    ///
    /// | channel | what | range |
    /// |---|---|---|
    /// | R | `round(255 · min(depth_mm / Layers::span_mm(), 1))` — how deep the wound is here | `0..=255` |
    /// | G | perceptual roughness of the exposed tissue, from `crate::cross_section::texel_at` | `0..=255` |
    /// | B | metallic — always `0` | `0` |
    /// | A | `255` where anything has been removed, `0` on intact surface | `0` or `255` |
    ///
    /// So `R · span_mm / 255` is the millimetres of tissue gone and `A` is the wound's own mask — the
    /// two things a caller's shader would otherwise have to infer from the colour, which cannot be
    /// done because muscle and a red shirt are the same red. They are still *output*: nothing in this
    /// crate reads a pixel back.
    ///
    /// **The material must set `perceptual_roughness: 1.0`**, because Bevy *multiplies* the scalar by
    /// the texture (`:157-163`) and the shipped scalar would scale the map away. That channel is
    /// carrying most of the effect: wet muscle and dry cortex differ far more in gloss than in hue,
    /// and specular wetness is the strongest disgust cue there is (Oum, Lieberman & Aylward,
    /// `doi:10.1080/02699931.2010.496997`).
    pub fn roughness(&self) -> Handle<Image> {
        self.rough_handle.clone()
    }

    /// The tangent-space normal image, linear RGBA8. Feed it to
    /// `StandardMaterial::normal_map_texture`, and give the mesh tangents
    /// (`Mesh::generate_tangents`); without them Bevy draws the surface flat.
    ///
    /// **The crater's relief, from the depth buffer.** Each texel's normal is the slope of the peeled
    /// surface, central differences of the removed depth over [`UV_SPAN_M`]'s millimetres per texel,
    /// in glTF's convention (`+Y` up the image, as Bevy expects with `flip_normal_map_y` off). Intact
    /// skin is flat, `(0, 0, 1)`. The walls tilt toward the crater's centre, so a light catches the
    /// far wall and leaves the near one in shade, and the wound reads as a hole rather than a
    /// painted disc.
    pub fn normal(&self) -> Handle<Image> {
        self.normal_handle.clone()
    }

    /// Edge length in texels.
    pub fn size(&self) -> u32 {
        self.size
    }

    /// **The bytes this canvas holds in the main world**, apart from its three images: the removed
    /// depth (`u16` a texel), the stamp mask, the grain-averaged tissue table, and the CPU copy of each
    /// image. See [`image_bytes`](Self::image_bytes) for what the images add.
    pub fn heap_bytes(&self) -> usize {
        self.depth.capacity() * std::mem::size_of::<u16>()
            + self.mask.capacity()
            + self.albedo_px.capacity()
            + self.rough_px.capacity()
            + self.normal_px.capacity()
            + self.tissue.capacity() * std::mem::size_of::<Option<([u8; 3], u8)>>()
    }

    /// **The size of the three images this canvas uploads** (albedo, metallic-roughness and normal),
    /// once. The main world keeps a copy of each and the GPU holds another, so the memory they cost
    /// is twice this.
    pub fn image_bytes(&self) -> usize {
        self.albedo_px.len() + self.rough_px.len() + self.normal_px.len()
    }

    /// Which body region's thickness row this canvas peels through.
    pub fn region(&self) -> Region {
        self.region
    }

    /// The thickness table this canvas was built with — what a caller needs to turn a depth back into
    /// a layer without keeping a second copy of the row.
    pub fn layers(&self) -> Layers {
        self.layers
    }

    /// **Tissue removed at a texel, millimetres.** `None` outside the canvas: a texel that does not
    /// exist has had nothing taken off it, and saying `0.0` would make an off-canvas read look like
    /// intact skin.
    pub fn depth_at(&self, x: u32, y: u32) -> Option<f32> {
        if x >= self.size || y >= self.size {
            return None;
        }
        self.depth
            .get((y as usize) * (self.size as usize) + x as usize)
            .map(|&d| d as f32 / HMM_PER_MM)
    }

    /// Texels whose depth has reached the cortex — the size of the exposed bone, in texels.
    ///
    /// Counts marrow too, because a texel that has been dug past the cortex is still bone at the
    /// surface as far as anything above this crate is concerned. For the finer question, ask
    /// [`exposed_area`](Self::exposed_area).
    pub fn bone_texels(&self) -> u32 {
        self.bone_texels
    }

    /// Whether the CPU state has diverged from the uploaded images.
    pub fn is_dirty(&self) -> bool {
        self.dirty_since.is_some()
    }

    /// **Heal every wound, keeping the canvas and its images.** The depth buffer, the three images'
    /// bytes and the bone handoff go back to what [`new`](Self::new) made, so [`digest`](Self::digest)
    /// is a fresh canvas's again and the next wound to reach the cortex hands bone off again; the
    /// images are re-uploaded on the next [`flush`](Self::flush), as of `tick`. Nothing is
    /// reallocated.
    pub fn clear(&mut self, tick: u32) {
        let (base, rough) = (self.base_rgba, [0, self.base_rough, 0, 0]);
        self.depth.fill(0);
        self.albedo_px.as_chunks_mut::<4>().0.fill(base);
        self.rough_px.as_chunks_mut::<4>().0.fill(rough);
        self.normal_px.as_chunks_mut::<4>().0.fill([128, 128, 255, 255]);
        self.lump.fill(0);
        // Every pixel is what a shade of an all-zero depth draws, so nothing is stale.
        self.stale = None;
        self.bone_texels = 0;
        self.bone_handed_off = false;
        self.painted_at = tick;
        self.dirty_since = Some(self.dirty_since.map_or(tick, |since| since.min(tick)));
    }

    /// The tick at which this canvas first diverged from its images, or `None` when they agree.
    ///
    /// The plugin's upload budget sorts on this — oldest dirty first — so a canvas that has been
    /// waiting cannot be starved by one that keeps being repainted.
    pub fn dirty_since(&self) -> Option<u32> {
        self.dirty_since
    }

    /// **Peel at a UV with the margin torn**: [`paint_uv`](Self::paint_uv), and for a `tear` over zero a **star of radial
    /// splits** laid over the crater, the way an exit wound is. `tear` is in `[0, 1]`; `0.0` (or a non-finite value) is
    /// `paint_uv` to the byte, which is how `paint_uv` is written.
    ///
    /// **What an exit wound is** (`docs/research/torn-exits.md`): where a projectile leaves a body the skin is not cut
    /// but burst outward from inside, and the margin is irregular and torn, often stellate: a small hub and thin
    /// radiating splits of different lengths, against the entrance's small round hole with a clean margin. Frances
    /// Glessner Lee's 1940 chest plates model both with a revolver's lead bullet (public domain, Wikimedia Commons:
    /// `Experimental_gunshot_exit_wound.jpg` against `Experimental_gunshot_entrance_wound_-_dist_to_target_25_inch.jpg`):
    /// the entrance a round hole and a few specks, the exit a hub of six or seven splits that run several hub radii.
    /// **What this paints** is that: the crater as `paint_uv` makes it, its rim shifted a little by two low harmonics of
    /// the angle (so it is not a circle), and `5 + 3 h` splits (`h` a hash of the place, five to seven), each at
    /// its own angle, `TEAR_SPLIT_WIDTH` texels wide at the root and tapering to a point, reaching from half a radius
    /// out to between `1.15` and `1.15 + TEAR_REACH · tear` radii from the centre, and `TEAR_SPLIT_DEPTH` of the depth
    /// of the crater, so a split opens the skin and shows what is under it and does not bore into the muscle.
    ///
    /// **Everything is a function of the place.** The angles, lengths and widths are `crate::bloodstain::hash_f32` of the
    /// centre texel, so the same hit in the same place tears the same way on every run and on every machine, and two
    /// hits a texel apart tear differently. The angle goes through `libm`, the one math path of this crate. Depth is added
    /// as for any crater, and the bone handoff fires as it does for any crater.
    pub fn paint_uv_torn(&mut self, uv: Vec2, radius_uv: f32, depth_mm: f32, tear: f32, tick: u32) -> Handoff {
        self.peel(uv, radius_uv, depth_mm, tear, tick)
    }

    /// **Peel at a UV.** Adds `depth_mm` of removed tissue at the centre, falling smoothly to nothing
    /// at `radius_uv`, and saturating at `Layers::span_mm()`.
    ///
    /// The falloff is `smoothstep(1 − d/r)` on the distance from the stamp centre, quantised to a byte
    /// and applied as `depth · w / 255` in integers. Smooth rather than a disc because a flat disc
    /// stacks into a cylinder — every repeat hit deepens a bore with a vertical wall, and the layer
    /// bands never show. With a falloff, repeated hits open a **crater**, and the crater's rim is
    /// exactly where the caller can read the bands off: skin at the edge, then fat, then muscle, then
    /// bone at the centre. That is the whole visual of a flaymap.
    ///
    /// UVs outside `[0, 1]`, a non-finite UV, and a non-finite radius or depth are **refused** with one
    /// warning per canvas and a [`Handoff`] that reports nothing. This differs from `wetmap`,
    /// which clamps a stray UV to the edge, and deliberately: blood on the wrong texel is a cosmetic
    /// error, whereas peeling a body's edge texels down to bone because a ray came back with a UV of
    /// `1.4` is a gameplay one — the caller would then get a bone handoff for a hit that never landed.
    pub fn paint_uv(&mut self, uv: Vec2, radius_uv: f32, depth_mm: f32, tick: u32) -> Handoff {
        self.peel(uv, radius_uv, depth_mm, 0.0, tick)
    }

    /// The one crater: [`paint_uv`](Self::paint_uv) at `tear` zero, [`paint_uv_torn`](Self::paint_uv_torn) over it.
    fn peel(&mut self, uv: Vec2, radius_uv: f32, depth_mm: f32, tear: f32, tick: u32) -> Handoff {
        let tear = if tear.is_finite() { tear.clamp(0.0, 1.0) } else { 0.0 };
        let refused = Handoff {
            deepest_layer: Layer::Skin,
            bone_reached: false,
            first_bone_uv: None,
            at: None,
            normal: None,
        };
        let ok_uv = uv.is_finite() && (0.0..=1.0).contains(&uv.x) && (0.0..=1.0).contains(&uv.y);
        if !ok_uv || !radius_uv.is_finite() || !depth_mm.is_finite() {
            if !self.warned_input {
                self.warned_input = true;
                warn!(
                    "flaymap: a paint at uv ({}, {}) with radius {radius_uv} and depth {depth_mm} mm \
                     is off the canvas or not a number, so nothing was peeled; this warns once",
                    uv.x, uv.y
                );
            }
            return refused;
        }
        if radius_uv <= 0.0 || depth_mm <= 0.0 {
            // Not bad input — a zero-depth or zero-radius hit is a hit that did nothing — so no
            // warning and no state change.
            return refused;
        }

        let n = self.size as i64;
        // Half the stamp, in texels, so the footprint is symmetric about its centre texel by
        // construction: an even edge length would put one more column on one side than the other and
        // a crater would drift by half a texel per hit. A torn crater is as wide as its longest split.
        let r_tex = radius_uv * self.size as f32;
        let cx = ((uv.x * self.size as f32) as i64).clamp(0, n - 1);
        let cy = ((uv.y * self.size as f32) as i64).clamp(0, n - 1);
        // A big crater's rim is out of round, and the stamp must reach as far as the rim does.
        let rim = if tear > 0.0 { 0.0 } else { rim_amplitude(r_tex) };
        let half = if tear > 0.0 { ((r_tex * (1.15 + TEAR_REACH * tear)).ceil() as i64).clamp(0, n) } else { ((r_tex * (1.0 + 3.0 * rim)).ceil() as i64).clamp(0, n) };
        let px = 2 * half + 1;
        let place = (cx as u32).wrapping_mul(0x9E37_79B1) ^ (cy as u32).wrapping_mul(0x85EB_CA6B);
        if tear > 0.0 {
            self.build_torn_mask(px, half, r_tex, tear, place);
        } else {
            self.build_mask(px, half, r_tex, rim, place);
        }

        let hmm = (depth_mm * HMM_PER_MM).round().clamp(0.0, u16::MAX as f32) as u32;
        let cap = self.cap_hmm();
        let cortex = self.cortex_hmm();

        let mut changed = false;
        let mut crossed = false;
        let mut deepest: Option<u16> = None;
        for my in 0..px {
            let y = cy + my - half;
            if y < 0 || y >= n {
                continue;
            }
            for mx in 0..px {
                let x = cx + mx - half;
                if x < 0 || x >= n {
                    continue;
                }
                let Some(&w) = self.mask.get((my * px + mx) as usize) else {
                    continue;
                };
                if w == 0 {
                    continue;
                }
                // In bounds by the two guards above; `get_mut` rather than `[]` so the crate holds no
                // panicking index at all.
                let Some(cell) = self.depth.get_mut((y * n + x) as usize) else {
                    continue;
                };
                let before = *cell;
                let after = (before as u32 + hmm * w as u32 / 255).min(cap as u32) as u16;
                if after != before {
                    *cell = after;
                    changed = true;
                    self.stale = Some(self.stale.map_or([x, y, x, y], |[x0, y0, x1, y1]| [x0.min(x), y0.min(y), x1.max(x), y1.max(y)]));
                }
                if before < cortex && after >= cortex {
                    self.bone_texels = self.bone_texels.saturating_add(1);
                    crossed = true;
                }
                deepest = Some(deepest.map_or(after, |d| d.max(after)));
            }
        }

        if changed {
            self.painted_at = tick;
            if self.dirty_since.is_none() {
                self.dirty_since = Some(tick);
            }
        }

        let bone_reached = crossed && !self.bone_handed_off;
        if bone_reached {
            self.bone_handed_off = true;
        }
        Handoff {
            deepest_layer: deepest
                .map_or(Layer::Skin, |d| self.layers.at(d as f32 / HMM_PER_MM).0),
            bone_reached,
            first_bone_uv: bone_reached.then_some(uv),
            // A UV is all this entry point was given. `paint_world` fills both in from its own ray.
            at: None,
            normal: None,
        }
    }

    /// **[`paint_world`](Self::paint_world) with the margin torn**: the same ray, the crater of
    /// [`paint_uv_torn`](Self::paint_uv_torn). `tear` zero is `paint_world` to the byte.
    #[allow(clippy::too_many_arguments)]
    pub fn paint_world_torn(
        &mut self,
        mesh: &Mesh,
        xf: &GlobalTransform,
        from: Vec3,
        dir: Vec3,
        radius_uv: f32,
        depth_mm: f32,
        tear: f32,
        tick: u32,
    ) -> Option<Handoff> {
        self.world(mesh, xf, from, dir, radius_uv, depth_mm, tear, tick)
    }

    /// **Peel where a world-space ray hits this actor.**
    ///
    /// Möller–Trumbore over the mesh's `ATTRIBUTE_POSITION` and index buffer, then a barycentric read
    /// of `ATTRIBUTE_UV_0`. The ray is moved into mesh space by one inverse transform rather than the
    /// geometry being moved into world space.
    ///
    /// Returns `None` when the ray misses, and `None` with **one warning per mesh** when the mesh
    /// cannot carry a flaymap at all — no `Float32x2` UV0, no `Float32x3` positions, or not a triangle
    /// list. That refusal is the point: a mesh without UVs makes every hit a caller lands peel
    /// nothing, and returning a `Handoff` after peeling nothing would hide it.
    pub fn paint_world(
        &mut self,
        mesh: &Mesh,
        xf: &GlobalTransform,
        from: Vec3,
        dir: Vec3,
        radius_uv: f32,
        depth_mm: f32,
        tick: u32,
    ) -> Option<Handoff> {
        self.world(mesh, xf, from, dir, radius_uv, depth_mm, 0.0, tick)
    }

    /// The one ray: [`paint_world`](Self::paint_world) at `tear` zero.
    #[allow(clippy::too_many_arguments)]
    fn world(
        &mut self,
        mesh: &Mesh,
        xf: &GlobalTransform,
        from: Vec3,
        dir: Vec3,
        radius_uv: f32,
        depth_mm: f32,
        tear: f32,
        tick: u32,
    ) -> Option<Handoff> {
        let inv = xf.affine().inverse();
        let origin = inv.transform_point3(from);
        let local_dir = inv.transform_vector3(dir);
        match ray_uv(mesh, origin, local_dir) {
            Pick::At { uv, point, normal } if (0.0..=1.0).contains(&uv.x) && (0.0..=1.0).contains(&uv.y) => {
                let mut handoff = self.peel(uv, radius_uv, depth_mm, tear, tick);
                handoff.at = Some(point);
                handoff.normal = Some(normal);
                Some(handoff)
            }
            // The mesh's own UV0 left the unit square at the hit. `paint_uv` would refuse it, but a
            // refusal dressed as a shallow hit with a point and a normal is indistinguishable from a
            // real one — so it is `None` here, and the mesh is named once, like any other mesh that
            // cannot carry a flaymap.
            Pick::At { uv, .. } => {
                self.warn_once(mesh, &format!(
                    "flaymap: this mesh's ATTRIBUTE_UV_0 leaves the unit square at the hit ({}, {}), so \
                     nothing was peeled; a flaymap atlas must be in [0, 1]",
                    uv.x, uv.y
                ));
                None
            }
            Pick::Miss => None,
            Pick::Unusable => {
                self.warn_once(
                    mesh,
                    "flaymap: this mesh has no Float32x2 ATTRIBUTE_UV_0 (or is not a triangle list), so it \
                     cannot carry a flaymap; nothing was peeled",
                );
                None
            }
        }
    }

    /// Warn about `mesh` once per canvas, whatever the reason: the memo is per mesh, not per message.
    fn warn_once(&mut self, mesh: &Mesh, message: &str) {
        let key = mesh_key(mesh);
        if self.warned.contains(&key) {
            return;
        }
        if self.warned.len() >= WARN_MEMO {
            self.warned.remove(0);
        }
        self.warned.push(key);
        warn!("{message}");
    }

    /// **Recolour the wound from the cross-section palette, and slope it.** The one pass that touches
    /// the pixels.
    ///
    /// Every texel with depth > 0 is the tissue `crate::cross_section` bakes a cut face's strip from,
    /// so a flayed patch and the stump beside it are the same tissue rather than two authored
    /// palettes that drift. Texels at depth 0 keep the intact surface's bytes exactly.
    ///
    /// **Filtered to the texel, because the grain is finer than one.** A texel is
    /// `UV_SPAN_M · scale.mm_per_unit / size` across — 7.8 mm on a 128 canvas — and fascicles are
    /// under one millimetre, lobules two. Point-sampling `texel_at` at that spacing put each texel on a
    /// different feature, and the crater came out as a disc of red and yellow dots. So a texel holds
    /// `texel_mean`, the tissue averaged over its grain, once a texel is twice the 2 mm grain or
    /// coarser; at the grain or finer it holds `texel_at` at its position, and between the two it
    /// blends. The mean is computed per 0.1 mm of depth on first use and kept.
    ///
    /// **The tissue is looked up at a ragged depth.** Depth is a smooth radial field, so read straight
    /// it banded a crater into concentric rings — a target, not a wound. Two octaves of value noise
    /// over the texel's millimetres, at most ±3 mm, tear the band edges and the rim. Only the colour
    /// sees it: the buffer, the digest, the R channel and the normals keep the true depth.
    ///
    /// The normal image is rewritten in the same pass. See [`normal`](Self::normal).
    ///
    /// **It redraws what changed** (0.5.59): the texels peeled since the last shade, and the normals
    /// one texel round them, which read their neighbours; the first shade, and a shade with another
    /// texel size, `tile_mm` or `seed`, redraws all of them. Every other pixel already shows the depth
    /// it holds, so the bytes are exactly those of a redraw of every texel.
    ///
    /// **Call it after the last paint of a tick and before `Update`**, where the plugin's budget will
    /// upload it. Calling it per paint costs the paint's footprint; not calling it at all leaves the
    /// images showing intact skin over a peeled buffer.
    pub fn shade(&mut self, s: &FlaySettings) {
        let per_texel = UV_SPAN_M * s.scale.mm_per_unit / self.size as f32;
        let mm = if per_texel.is_finite() { per_texel } else { 0.0 };
        let size = self.size as usize;
        let n = size as i64;
        let key = (mm.to_bits(), s.tile_mm.to_bits(), s.seed);
        let [x0, y0, x1, y1] = if self.shaded_for == Some(key) {
            match self.stale.take() {
                Some(r) => r,
                None => return,
            }
        } else {
            self.stale = None;
            [0, 0, n - 1, n - 1]
        };
        self.shaded_for = Some(key);
        let detail = {
            let t = ((2.0 * GRAIN_MM - mm) / GRAIN_MM).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        let key = (s.tile_mm.to_bits(), s.seed);
        if self.tissue_key != key {
            self.tissue.clear();
            self.tissue_key = key;
        }
        // The depth fraction's denominator: the whole span the layer table describes, which is also
        // what the buffer saturates at. `max` so a degenerate table cannot divide by zero.
        let span = self.layers.span_mm().max(1.0e-3);
        let (layers, depth, tissue, lump) = (&self.layers, &self.depth, &mut self.tissue, &mut self.lump);
        let (base_rgba, base_rough) = (self.base_rgba, self.base_rough);
        let tile = (size as f32 * mm / RAGGED_CELL_MM).max(1.0);
        // **The floor's lumps need texels smaller than they are**: full strength under half a cell, none from 0.85 of a cell, smooth between (a 256 canvas is 3.9 mm a texel, a 128 one 7.8).
        let lump_scale = {
            let t = ((0.85 * LUMP_CELL_MM - mm) / (0.35 * LUMP_CELL_MM)).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        let lump_tile = (size as f32 * mm / LUMP_CELL_MM).max(1.0);
        // Bone is smooth ivory and a pit of marrow is smooth blood: the lumps stop where the cortex starts.
        let bone_from = layers.starts_mm().get(3).copied().unwrap_or(f32::INFINITY);
        let mut changed = false;
        for (y, x) in (y0..=y1).flat_map(|y| (x0..=x1).map(move |x| (y, x))) {
            let i = (y * n + x) as usize;
            let (Some(&d), Some(a), Some(r)) = (depth.get(i), self.albedo_px.get_mut(i * 4..i * 4 + 4), self.rough_px.get_mut(i * 4..i * 4 + 4)) else {
                continue;
            };
            let (want_a, want_r) = if d == 0 {
                if let Some(l) = lump.get_mut(i) {
                    *l = 0;
                }
                (base_rgba, [0, base_rough, 0, 0])
            } else {
                let depth_mm = d as f32 / HMM_PER_MM;
                let (xm, ym) = ((i % size) as f32 * mm, (i / size) as f32 * mm);
                let coarse = value_noise(xm / RAGGED_CELL_MM, ym / RAGGED_CELL_MM, tile, s.seed ^ 0x7A6E);
                let fine = value_noise(xm / (RAGGED_CELL_MM * 0.4), ym / (RAGGED_CELL_MM * 0.4), tile * 2.5, s.seed ^ 0x3E11);
                let ragged = ((coarse - 0.5) * 1.4 + (fine - 0.5) * 0.6) * RAGGED_MM;
                // **The skin's lip**: the rim of the wound, where the depth has not yet reached `LIP_MM`, `1` at the edge and `0` by `LIP_MM`.
                let lip = {
                    // Thicker in one place and a thread in the next: the same slow noise that tears the bands.
                    let q = (1.0 - depth_mm / (LIP_MM * (0.55 + 0.9 * coarse))).clamp(0.0, 1.0);
                    q * q * (3.0 - 2.0 * q)
                };
                // **The floor's lumps**: a nodule stands up (shallower, paler, wetter), a crevice sinks (deeper, darker, rougher), and the
                // skin's lip, which is not yet `LUMP_FADE_MM` deep, stays smooth. `lumpy` is the height, in units of the lump, `0` where there are none.
                let k = lump_scale
                    * {
                        let q = (depth_mm / LUMP_FADE_MM).clamp(0.0, 1.0);
                        q * q * (3.0 - 2.0 * q)
                    }
                    * {
                        let q = ((depth_mm - (bone_from - 1.0)) / 4.0).clamp(0.0, 1.0);
                        1.0 - q * q * (3.0 - 2.0 * q)
                    };
                let (nod, seam) = if k > 0.0 { lumps(xm, ym, lump_tile, s.seed) } else { (0.0, 0.0) };
                let lumpy = if k > 0.0 { k * (nod - 0.8 * seam - 0.35) } else { 0.0 };
                if let Some(l) = lump.get_mut(i) {
                    *l = ((LUMP_RELIEF_MM * lumpy + LIP_RELIEF_MM * lip) * 20.0).round().clamp(-127.0, 127.0) as i8;
                }
                // Kept above zero so a peeled texel never shades as unpeeled skin.
                let look_mm = (depth_mm + ragged - LUMP_LOOK_MM * lumpy).max(0.01);
                let look = (look_mm * HMM_PER_MM).round().clamp(1.0, u16::MAX as f32) as u16;
                let bin = (look / TISSUE_BIN_HMM) as usize;
                if tissue.len() <= bin {
                    tissue.resize(bin + 1, None);
                }
                let mean = match tissue.get(bin).copied().flatten() {
                    Some(m) => m,
                    None => {
                        let at = (bin as u16 * TISSUE_BIN_HMM) as f32 / HMM_PER_MM;
                        let (c, rough) = texel_mean(layers, at, s.tile_mm, s.seed);
                        let m = ([enc(c[0]), enc(c[1]), enc(c[2])], enc(rough));
                        if let Some(slot) = tissue.get_mut(bin) {
                            *slot = Some(m);
                        }
                        m
                    }
                };
                let (c, rough) = if detail > 0.0 {
                    let (c, rough) = texel_at(layers, look_mm, xm, ym, s.tile_mm, s.seed);
                    let mix = |m: u8, p: f32| enc(m as f32 / 255.0 + (p - m as f32 / 255.0) * detail);
                    ([mix(mean.0[0], c[0]), mix(mean.0[1], c[1]), mix(mean.0[2], c[2])], mix(mean.1, rough))
                } else {
                    mean
                };
                let (c, rough) = if k > 0.0 {
                    let dark = (1.0 - LUMP_DARK * k * seam) * (1.0 + LUMP_LIGHT * k * nod * nod);
                    let pale = LUMP_PALE * k * nod;
                    let tint = |ch: u8, n: f32| {
                        let v = (ch as f32 / 255.0 * dark).min(1.0);
                        enc(v + (n - v) * pale)
                    };
                    let wet = rough as f32 / 255.0 * (1.0 - LUMP_WET * k * nod) + 0.25 * k * seam;
                    ([tint(c[0], LUMP_NODULE[0]), tint(c[1], LUMP_NODULE[1]), tint(c[2], LUMP_NODULE[2])], enc(wet))
                } else {
                    (c, rough)
                };
                // The lip over whatever the rim texel was: the skin's cut edge, paler than the skin round it.
                let c = if lip > 0.0 {
                    let t = LIP_PALE * lip;
                    let mix = |m: u8, p: f32| enc(m as f32 / 255.0 + (p - m as f32 / 255.0) * t);
                    [mix(c[0], LIP_COLOUR[0]), mix(c[1], LIP_COLOUR[1]), mix(c[2], LIP_COLOUR[2])]
                } else {
                    c
                };
                // R and A are the depth buffer written where a shader can read it, per
                // [`roughness`](Self::roughness): the true depth as a fraction of the table's span,
                // and the wound's own mask.
                ([c[0], c[1], c[2], 255], [enc(depth_mm / span), rough, 0, 255])
            };
            // Compare before writing: a canvas whose depths did not move must not ask for an upload,
            // because a still scene should cost no bandwidth.
            for (slot, want) in a.iter_mut().zip(want_a) {
                if *slot != want {
                    *slot = want;
                    changed = true;
                }
            }
            for (slot, want) in r.iter_mut().zip(want_r) {
                if *slot != want {
                    *slot = want;
                    changed = true;
                }
            }
        }
        // The slope of the peeled surface. `h = -depth`, so with glTF's `+Y` up the image (towards
        // row 0) the normal is `(-dh/du, dh/dv, 1)` over twice a texel's millimetres. One texel wider
        // than the colours, since a normal reads the depth either side of it.
        let two_mm = (2.0 * mm).max(1.0e-3);
        let lump = &*lump;
        // The height the normal sees: the depth less the lump's own height (a raised lump is a shallower texel).
        let at = |x: i64, y: i64| -> f32 {
            let (x, y) = (x.clamp(0, n - 1), y.clamp(0, n - 1));
            let j = (y * n + x) as usize;
            depth.get(j).map_or(0.0, |&d| d as f32 / HMM_PER_MM) - lump.get(j).map_or(0.0, |&l| l as f32 / 20.0)
        };
        let (nx0, ny0, nx1, ny1) = ((x0 - 1).max(0), (y0 - 1).max(0), (x1 + 1).min(n - 1), (y1 + 1).min(n - 1));
        for (y, x) in (ny0..=ny1).flat_map(|y| (nx0..=nx1).map(move |x| (y, x))) {
            let i = (y * n + x) as usize;
            let Some(px) = self.normal_px.get_mut(i * 4..i * 4 + 4) else { continue };
            let dx = (at(x + 1, y) - at(x - 1, y)) / two_mm;
            let dy = -(at(x, y + 1) - at(x, y - 1)) / two_mm;
            let want = if dx == 0.0 && dy == 0.0 {
                [128, 128, 255, 255]
            } else {
                let k = 1.0 / (dx * dx + dy * dy + 1.0).sqrt();
                [enc(dx * k * 0.5 + 0.5), enc(dy * k * 0.5 + 0.5), enc(k * 0.5 + 0.5), 255]
            };
            for (slot, w) in px.iter_mut().zip(want) {
                if *slot != w {
                    *slot = w;
                    changed = true;
                }
            }
        }
        if changed && self.dirty_since.is_none() {
            self.dirty_since = Some(self.painted_at);
        }
    }

    /// **Upload, and the only place `Assets<Image>` is touched.**
    ///
    /// Returns `false` and writes nothing when the canvas is clean, so a still scene costs no
    /// bandwidth. The per-frame budget *across* canvases is the plugin's — see
    /// [`FlaySettings::max_canvas_updates_per_tick`].
    ///
    /// Also returns `false` without clearing the dirty flag if any image has gone or has been
    /// resized under it, so the next flush retries rather than the canvas quietly diverging forever.
    pub fn flush(&mut self, images: &mut Assets<Image>) -> bool {
        if self.dirty_since.is_none() {
            return false;
        }
        let ok = upload(images, &self.albedo_handle, &self.albedo_px)
            && upload(images, &self.rough_handle, &self.rough_px)
            && upload(images, &self.normal_handle, &self.normal_px);
        if ok {
            self.dirty_since = None;
        }
        ok
    }

    /// **FNV-1a over the depth buffer** — each texel's hundredths of a millimetre, little-endian,
    /// row-major.
    ///
    /// Over the *buffer*, not the images: see [`crate::flaymap::digest`] for why the CPU state is the authority
    /// and the uploaded pixels are not. [`shade`](Self::shade) is a pure function of this buffer, so
    /// folding the pixels as well would hash the same information twice and would make a palette
    /// change read as a simulation divergence.
    pub fn digest(&self) -> u64 {
        let mut f = Fnv1a::new();
        for &d in &self.depth {
            f.u16(d);
        }
        f.finish()
    }

    /// **How much of this canvas is currently showing a given tissue**, in texels.
    ///
    /// The layer at each texel's own depth, counted. An untouched texel is at depth 0 and so counts as
    /// [`Layer::Skin`] — which is the honest answer: intact skin is exposed skin, and a caller asking
    /// how much skin is showing wants the unbroken part included. `exposed_area(Layer::Cortex)` is
    /// therefore the interesting one: it is zero until something has been dug to bone.
    pub fn exposed_area(&self, layer: Layer) -> u32 {
        let mut n = 0;
        for &d in &self.depth {
            if self.layers.at(d as f32 / HMM_PER_MM).0 == layer {
                n += 1;
            }
        }
        n
    }

    /// The depth buffer's ceiling, hundredths of a millimetre: the whole span the layer table
    /// describes. Peeling past marrow is not modelled, so the buffer saturates here.
    fn cap_hmm(&self) -> u16 {
        (self.layers.span_mm() * HMM_PER_MM).round().clamp(0.0, u16::MAX as f32) as u16
    }

    /// The top of the cortex, hundredths of a millimetre — the one threshold [`Handoff`] is about.
    fn cortex_hmm(&self) -> u16 {
        let starts = self.layers.starts_mm();
        let cortex = starts.get(3).copied().unwrap_or(f32::INFINITY);
        (cortex * HMM_PER_MM).round().clamp(0.0, u16::MAX as f32) as u16
    }

    /// Fill [`mask`](Self::mask) with a `px × px` **torn crater**: [`build_mask`](Self::build_mask)'s falloff with its
    /// rim moved in and out by two low harmonics of the angle, and, laid over it by `max`, `5 + 3 h` thin splits that
    /// reach out from half a radius and taper to a point. Every number is a hash of `seed` (a place); the angles go
    /// through `libm`; the weights are bytes. See [`paint_uv_torn`](Self::paint_uv_torn).
    fn build_torn_mask(&mut self, px: i64, half: i64, r_tex: f32, tear: f32, seed: u32) {
        let need = (px * px).max(0) as usize;
        if self.mask.len() != need {
            self.mask.resize(need, 0);
        }
        let r = r_tex.max(1.0e-6);
        let h = |salt: u32| crate::bloodstain::hash_f32(seed ^ salt.wrapping_mul(0x27D4_EB2F));
        let tau = core::f32::consts::TAU;
        // The rim: `1 + tear (a2 cos(2 th + p2) + a3 cos(3 th + p3))`, a few per cent out of round each way.
        let (p2, p3) = (h(0x11) * tau, h(0x12) * tau);
        let (a2, a3) = (0.10 * tear * (0.5 + 0.5 * h(0x13)), 0.08 * tear * (0.5 + 0.5 * h(0x14)));
        // The splits: five to seven, each at its own angle, reach and width.
        const MAX_SPLITS: usize = 7;
        let count = (5 + (h(0x21) * 3.0) as usize).min(MAX_SPLITS);
        let phase = h(0x22) * tau;
        // (cos, sin) of the split's angle, its tip and root width, how far its tip is bent off the straight, and the ripple.
        let mut splits = [(0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32); MAX_SPLITS];
        for (i, split) in splits.iter_mut().enumerate().take(count) {
            let k = (i as u32 + 1) << 8;
            let angle = phase + (i as f32 + 0.7 * (h(k | 1) - 0.5)) * tau / count as f32;
            // Squared, so a few splits are short and most are long: the lengths of Glessner Lee's exit plate run from two to nine hubs.
            let long = h(k | 2);
            let tip = r * (1.05 + TEAR_REACH * tear * (0.15 + 0.85 * long * long));
            let root = (0.7 + 0.6 * h(k | 3)) * TEAR_SPLIT_WIDTH;
            let bend = (h(k | 4) - 0.5) * 0.5 * tip;
            let ripple = 0.3 + 0.5 * h(k | 5);
            *split = (libm::cosf(angle), libm::sinf(angle), tip, root, bend, ripple, h(k | 6) * tau);
        }
        let from = 0.5 * r;
        // Past the farthest the rim can reach there is no crater to find the angle of: the splits are all that is left.
        let rim_max = r * (1.0 + a2 + a3);
        let byte = |w: f32| (w * 255.0).round().clamp(0.0, 255.0) as u8;
        self.mask.fill(0);
        // The crater, over the square its rim can reach.
        let reach = rim_max.ceil() as i64 + 1;
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let (mx, my) = (half + dx, half + dy);
                if mx < 0 || my < 0 || mx >= px || my >= px {
                    continue;
                }
                let (fx, fy) = (dx as f32, dy as f32);
                let d = (fx * fx + fy * fy).sqrt();
                if d >= rim_max {
                    continue;
                }
                let th = libm::atan2f(fy, fx);
                let rim = r * (1.0 + a2 * libm::cosf(2.0 * th + p2) + a3 * libm::cosf(3.0 * th + p3));
                let t = (1.0 - d / rim.max(1.0e-6)).clamp(0.0, 1.0);
                if let Some(slot) = self.mask.get_mut((my * px + mx) as usize) {
                    *slot = byte(profile(t));
                }
            }
        }
        // The splits, each over the box that holds its bent, rippled axis and its width, laid over the crater by the larger byte
        // (rounding is monotone, so this is the byte of the larger weight).
        for &(c, s, tip, root, bend, ripple, ph) in splits.iter().take(count) {
            let margin = bend.abs() + ripple + root + 1.0;
            let (x0, x1) = (((c * from).min(c * tip) - margin).floor() as i64, ((c * from).max(c * tip) + margin).ceil() as i64);
            let (y0, y1) = (((s * from).min(s * tip) - margin).floor() as i64, ((s * from).max(s * tip) + margin).ceil() as i64);
            for dy in y0.max(-half)..=y1.min(half) {
                for dx in x0.max(-half)..=x1.min(half) {
                    let (fx, fy) = (dx as f32, dy as f32);
                    let along = fx * c + fy * s;
                    if along < from || along > tip {
                        continue;
                    }
                    let u = (along - from) / (tip - from).max(1.0e-6);
                    let width = (root * (1.0 - 0.8 * u)).max(0.3);
                    // A split is not a ray: it bends toward its tip and ripples a little, as a tear does. A texel farther from
                    // the bent axis than the width and the whole ripple is not on the split, and is not asked for its sine.
                    let (line, drift) = (fy * c - fx * s, bend * u * u);
                    if (line - drift).abs() > width + ripple + 1.0e-3 {
                        continue;
                    }
                    let across = (line - (drift + ripple * libm::sinf(3.8 * u + ph))).abs();
                    let q = (1.0 - across / width).clamp(0.0, 1.0);
                    let split = byte(TEAR_SPLIT_DEPTH * (q * q * (3.0 - 2.0 * q)) * (1.0 - u * u));
                    if let Some(slot) = self.mask.get_mut(((half + dy) * px + half + dx) as usize) {
                        *slot = (*slot).max(split);
                    }
                }
            }
        }
    }

    /// Fill [`mask`](Self::mask) with a `px × px` radial falloff: 255 at the centre texel, 0 at the rim.
    ///
    /// **The profile is [`profile`]**: a wall over the outer [`WALL`] of the radius and a pit that keeps
    /// deepening to the centre ([`PIT`]), where 0.8.7's was a smoothstep over the whole radius. The rim is the
    /// radius times `1 + rim · (cos(3θ + p3) + cos(5θ + p5) + cos(8θ + p8))`, every phase and weight a hash of
    /// `seed` (a place), so a big crater's margin is out of round and the same hit in the same place tears the
    /// same on every machine; `rim` is [`rim_amplitude`], zero for a small crater, which is round to the byte.
    ///
    /// Quantised to a byte so the depth a texel receives is an integer multiplication rather than a float
    /// accumulation — the property that lets the buffer be hashed at all.
    fn build_mask(&mut self, px: i64, half: i64, r_tex: f32, rim: f32, seed: u32) {
        let need = (px * px).max(0) as usize;
        if self.mask.len() != need {
            self.mask.resize(need, 0);
        }
        let r = r_tex.max(1.0e-6);
        let h = |salt: u32| crate::bloodstain::hash_f32(seed ^ salt.wrapping_mul(0x27D4_EB2F));
        let tau = core::f32::consts::TAU;
        let (p3, p5, p8) = (h(0x31) * tau, h(0x32) * tau, h(0x33) * tau);
        let (w3, w5, w8) = (0.5 + 0.5 * h(0x34), 0.5 + 0.5 * h(0x35), 0.5 + 0.5 * h(0x36));
        for my in 0..px {
            for mx in 0..px {
                let (dx, dy) = ((mx - half) as f32, (my - half) as f32);
                let d = (dx * dx + dy * dy).sqrt();
                let edge = if rim > 0.0 {
                    let th = libm::atan2f(dy, dx);
                    r * (1.0 + rim * (w3 * libm::cosf(3.0 * th + p3) + w5 * libm::cosf(5.0 * th + p5) + w8 * libm::cosf(8.0 * th + p8)))
                } else {
                    r
                };
                let t = (1.0 - d / edge.max(1.0e-6)).clamp(0.0, 1.0);
                if let Some(slot) = self.mask.get_mut((my * px + mx) as usize) {
                    *slot = (profile(t) * 255.0).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
}

/// **The crater's falloff**, `t = 1 - d / rim` in `[0, 1]` to a weight in `[0, 1]`: `(1 - PIT) S(min(t / WALL, 1)) + PIT S(t)` with `S` the smoothstep. See [`WALL`] and [`PIT`].
fn profile(t: f32) -> f32 {
    let s = |x: f32| x * x * (3.0 - 2.0 * x);
    (1.0 - PIT) * s((t / WALL).clamp(0.0, 1.0)) + PIT * s(t.clamp(0.0, 1.0))
}

/// **How out of round a plain crater's rim is**, as a share of its radius per harmonic: zero under [`RIM_FROM_TEXELS`] of radius, [`RIM_RAG`] from [`RIM_FULL_TEXELS`], smooth between.
fn rim_amplitude(r_tex: f32) -> f32 {
    let t = ((r_tex - RIM_FROM_TEXELS) / (RIM_FULL_TEXELS - RIM_FROM_TEXELS)).clamp(0.0, 1.0);
    RIM_RAG * t * t * (3.0 - 2.0 * t)
}

/// **The floor's lumps at one place**: `(nodule, seam)`, each in `[0, 1]`. A Worley field at [`LUMP_CELL_MM`] and a second octave at half that, `7 : 3`. The nodule is a dome that is `1` at a cell's feature
/// point and `0` from `0.8` of a cell away; the seam is `1` where the nearest and second-nearest points are equally far and `0` once they differ by a quarter of a cell. A pure function of the
/// position in millimetres and the seed, periodic along `y` with `tile` cells (the canvas's own span, which has no seam: UV 0 and 1 are different places).
fn lumps(xm: f32, ym: f32, tile: f32, seed: u32) -> (f32, f32) {
    let dome = |f: f32| {
        let q = (1.0 - f / 0.8).clamp(0.0, 1.0);
        q * q * (3.0 - 2.0 * q)
    };
    let seam = |f1: f32, f2: f32| {
        let q = ((0.25 - (f2 - f1)) / 0.25).clamp(0.0, 1.0);
        q * q * (3.0 - 2.0 * q)
    };
    // The lattice is warped by a slow noise so the cells are not a grid, and a second, coarser one stands up as the big nodules of fat among the small.
    let warp = |a: f32, b: f32, salt: u32| (value_noise(a / 30.0, b / 30.0, (tile * LUMP_CELL_MM / 30.0).max(1.0), seed ^ salt) - 0.5) * 12.0;
    let (wx, wy) = (xm + warp(xm, ym, 0xA1), ym + warp(xm + 91.0, ym + 53.0, 0xB2));
    let (f1, f2) = worley(wx / LUMP_CELL_MM, wy / LUMP_CELL_MM, tile, seed ^ 0x1B3D);
    let (g1, g2) = worley(wx / (2.3 * LUMP_CELL_MM), wy / (2.3 * LUMP_CELL_MM), (tile / 2.3).max(1.0), seed ^ 0x2C4E);
    // Nodules come in patches: a slow field (`LUMP_PATCH_MM`) scales how many of them are pale, so a wound has a cream cluster here and a dark pit there, and not one even pebbling.
    let patch = 0.35 + 1.3 * value_noise(xm / LUMP_PATCH_MM, ym / LUMP_PATCH_MM, (tile * LUMP_CELL_MM / LUMP_PATCH_MM).max(1.0), seed ^ 0x77A1);
    // And a broad swell under all of it (`55 mm`), so the floor is a torn mass with folds and not an even pebbling.
    let swell = value_noise(xm / 55.0, ym / 55.0, (tile * LUMP_CELL_MM / 55.0).max(1.0), seed ^ 0x5E11) - 0.5;
    ((((0.65 * dome(f1) + 0.45 * dome(g1)) * patch) + 0.7 * swell).clamp(0.0, 1.0), (0.65 * seam(f1, f2) + 0.35 * seam(g1, g2)).clamp(0.0, 1.0))
}

/// A `[0, 1]` channel as a byte.
#[inline]
fn enc(v: f32) -> u8 {
    let v = if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 };
    (v * 255.0).round() as u8
}

/// Copy a pixel buffer into an image, refusing a size mismatch rather than half-writing it.
fn upload(images: &mut Assets<Image>, handle: &Handle<Image>, px: &[u8]) -> bool {
    // `mut` because `Assets::get_mut` hands back a change-detecting guard by value, not a `&mut`.
    let Some(mut image) = images.get_mut(handle) else {
        return false;
    };
    let Some(data) = image.data.as_mut() else {
        return false;
    };
    if data.len() != px.len() {
        return false;
    }
    data.copy_from_slice(px);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The limb row: 1.9 mm skin, 7.2 fat, 18.7 muscle, 5.0 cortex, 8.0 drawn marrow.
    fn limb() -> Layers {
        Layers::for_region(Region::Limb)
    }

    fn scratch(size: u32) -> (Assets<Image>, FlayCanvas) {
        let mut images = Assets::<Image>::default();
        let canvas =
            FlayCanvas::new(&mut images, size, Region::Limb, limb(), [0.78, 0.66, 0.60], 0.55);
        (images, canvas)
    }

    /// The centre texel of a canvas painted at `(0.5, 0.5)`.
    fn centre(canvas: &FlayCanvas) -> (u32, u32) {
        let c = canvas.size() / 2;
        (c, c)
    }

    #[test]
    fn depth_never_decreases() {
        let (_images, mut canvas) = scratch(32);
        let uv = Vec2::new(0.5, 0.5);
        let (cx, cy) = centre(&canvas);
        let cap = limb().span_mm();

        let mut last = 0.0_f32;
        let mut saturated = false;
        for hit in 0..10 {
            canvas.paint_uv(uv, 0.2, 6.0, hit);
            let now = canvas.depth_at(cx, cy).unwrap_or(-1.0);
            assert!(now >= last, "hit {hit}: depth went backwards, {last} -> {now}");
            if !saturated {
                // Ten 6 mm hits is 60 mm against a 40.8 mm limb, so the last few are the saturated
                // ones and only the earlier ones are required to move.
                assert!(now > last, "hit {hit}: an unsaturated hit peeled nothing, still {now}");
            }
            saturated = now >= cap - 1.0e-3;
            last = now;
        }
        assert!(saturated, "ten 6 mm hits must reach the bottom of a 40.8 mm limb");
        assert!((last - cap).abs() < 0.01, "saturation must stop exactly at the span, got {last}");
    }

    #[test]
    fn the_handoff_fires_exactly_when_the_cortex_is_reached() {
        let (_images, mut canvas) = scratch(32);
        let uv = Vec2::new(0.5, 0.5);
        let cortex = limb().starts_mm()[3];

        // Half a millimetre short of bone: deep in muscle, no handoff.
        let first = canvas.paint_uv(uv, 0.2, cortex - 0.5, 0);
        assert_eq!(first.deepest_layer, Layer::Muscle);
        assert!(!first.bone_reached, "muscle is not bone");
        assert_eq!(first.first_bone_uv, None);
        assert_eq!(canvas.exposed_area(Layer::Cortex), 0, "nothing has reached the cortex yet");
        assert_eq!(canvas.bone_texels(), 0);

        // One more millimetre crosses it.
        let second = canvas.paint_uv(uv, 0.2, 1.0, 1);
        assert!(second.bone_reached, "crossing starts_mm()[3] is the handoff");
        assert_eq!(second.first_bone_uv, Some(uv));
        assert_eq!(second.at, None, "paint_uv was handed a UV, not a point");
        assert_eq!(second.normal, None, "and it has no triangle to take a normal from");
        assert!(canvas.exposed_area(Layer::Cortex) > 0, "bone must be visible once it is reached");
        assert!(canvas.bone_texels() > 0);

        // And it never fires again, however much more is taken off.
        let third = canvas.paint_uv(uv, 0.2, 4.0, 2);
        assert!(!third.bone_reached, "the handoff is once per canvas");
        assert_eq!(third.first_bone_uv, None);
    }

    #[test]
    fn a_paint_off_the_canvas_is_refused() {
        let (_images, mut canvas) = scratch(16);
        let before = canvas.digest();
        for bad in [
            Vec2::new(1.4, 0.5),
            Vec2::new(-0.01, 0.5),
            Vec2::new(0.5, f32::NAN),
            Vec2::new(f32::INFINITY, 0.0),
        ] {
            let h = canvas.paint_uv(bad, 0.2, 30.0, 0);
            assert_eq!(h.deepest_layer, Layer::Skin, "a refused paint reached nothing");
            assert!(!h.bone_reached);
            assert_eq!(h.first_bone_uv, None);
            assert_eq!((h.at, h.normal), (None, None));
        }
        // A NaN radius and a NaN depth are refused too, and neither panics.
        canvas.paint_uv(Vec2::new(0.5, 0.5), f32::NAN, 30.0, 0);
        canvas.paint_uv(Vec2::new(0.5, 0.5), 0.2, f32::NAN, 0);
        assert_eq!(canvas.digest(), before, "a refused paint must change nothing");
        assert_eq!(canvas.dirty_since(), None, "a refused paint must not ask for an upload");
    }

    /// The added half of the handoff contract: a world hit carries **where** it landed and the plane
    /// it landed on, so a consumer fracturing the exposed bone does not have to invert a UV.
    #[test]
    fn a_world_hit_carries_its_point_and_its_normal() {
        use bevy::asset::RenderAssetUsages;
        use bevy::mesh::{Indices, PrimitiveTopology};

        // One triangle in the z = 0 plane, unit UVs at its corners.
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
        mesh.insert_indices(Indices::U32(vec![0, 1, 2]));

        let (_images, mut canvas) = scratch(32);
        let xf = GlobalTransform::default();
        let hit = canvas
            .paint_world(&mesh, &xf, Vec3::new(0.25, 0.25, 1.0), -Vec3::Z, 0.2, 30.0, 0)
            .expect("a ray straight through the triangle must land");
        let at = hit.at.expect("a world hit knows where it landed");
        assert!(at.distance(Vec3::new(0.25, 0.25, 0.0)) < 1.0e-4, "point was {at}");
        let n = hit.normal.expect("a world hit knows what it landed on");
        assert!(n.distance(Vec3::Z) < 1.0e-4, "normal was {n}");
        assert!(hit.bone_reached, "30 mm through a 27.8 mm limb is bone");
        assert_eq!(hit.first_bone_uv, Some(Vec2::new(0.25, 0.25)));

        // A ray that goes past the triangle is not a hit at all, and says so with `None` rather than
        // with a handoff that reached nothing.
        assert!(
            canvas
                .paint_world(&mesh, &xf, Vec3::new(9.0, 9.0, 1.0), -Vec3::Z, 0.2, 30.0, 1)
                .is_none()
        );
    }

    /// **Shading what changed is shading everything, to the byte.** Two canvases take the same
    /// craters — overlapping, at an edge, deep enough to reach bone — each followed by a shade; one
    /// shades as it does, the other is made to redraw every texel every time. Their three images and
    /// their dirty ticks agree after every shade, through a change of palette and back, and a clear.
    #[test]
    fn shading_what_changed_is_shading_everything() {
        let (_ia, mut fast) = scratch(48);
        let (_ib, mut full) = scratch(48);
        let s = FlaySettings::default();
        let other = FlaySettings { seed: 7, tile_mm: 3.0, ..FlaySettings::default() };
        let hits = [(0.3, 0.4, 0.1, 6.0), (0.35, 0.42, 0.08, 30.0), (0.02, 0.97, 0.12, 9.0), (0.8, 0.2, 0.05, 2.0), (0.5, 0.5, 0.3, 40.0), (0.52, 0.48, 0.04, 3.0)];
        let both = |k: u32, uv: Vec2, r: f32, d: f32, settings: &FlaySettings, fast: &mut FlayCanvas, full: &mut FlayCanvas| {
            fast.paint_uv(uv, r, d, k);
            full.paint_uv(uv, r, d, k);
            fast.shade(settings);
            full.shaded_for = None;
            full.shade(settings);
            assert_eq!((&fast.albedo_px, &fast.rough_px), (&full.albedo_px, &full.rough_px), "hit {k}: the colours parted");
            assert_eq!(fast.normal_px, full.normal_px, "hit {k}: the normals parted");
            assert_eq!(fast.dirty_since, full.dirty_since, "hit {k}");
        };
        for (k, &(u, v, r, d)) in hits.iter().enumerate() {
            both(k as u32, Vec2::new(u, v), r, d, if k == 3 { &other } else { &s }, &mut fast, &mut full);
        }
        fast.clear(9);
        full.clear(9);
        both(10, Vec2::new(0.6, 0.6), 0.1, 8.0, &s, &mut fast, &mut full);
    }

    #[test]
    fn the_digest_is_over_depth_not_pixels() {
        let (_images, mut canvas) = scratch(24);
        let s = FlaySettings::default();
        canvas.paint_uv(Vec2::new(0.4, 0.6), 0.25, 12.0, 0);
        canvas.shade(&s);
        let once = canvas.digest();
        canvas.shade(&s);
        assert_eq!(canvas.digest(), once, "shading is not state");

        // A different palette moves the pixels and must not move the digest.
        let other = FlaySettings { seed: 7, tile_mm: 3.0, ..FlaySettings::default() };
        canvas.shade(&other);
        assert_eq!(canvas.digest(), once, "the digest folds tissue removed, not tissue colour");

        // Two canvases, identical script, identical digest.
        let (_i2, mut twin) = scratch(24);
        twin.paint_uv(Vec2::new(0.4, 0.6), 0.25, 12.0, 0);
        assert_eq!(twin.digest(), once);

        // And one texel of difference is visible in the fold.
        let (_i3, mut moved) = scratch(24);
        moved.paint_uv(Vec2::new(0.4 + 1.0 / 24.0, 0.6), 0.25, 12.0, 0);
        assert_ne!(moved.digest(), once, "a one-texel move must reach the digest");
    }

    #[test]
    fn a_coarse_canvas_shows_the_grain_averaged_tissue() {
        let (_images, mut canvas) = scratch(32);
        let s = FlaySettings::default();
        canvas.paint_uv(Vec2::new(0.5, 0.5), 0.2, 25.0, 0);
        canvas.shade(&s);

        let (cx, cy) = centre(&canvas);
        let depth = canvas.depth_at(cx, cy).unwrap_or(0.0);
        assert!(depth > 20.0, "the centre should be deep in muscle, got {depth} mm");

        // 31 mm texels are far coarser than the grain, so the texel is the tissue's mean at one
        // 0.1 mm bin — at the true depth torn by at most `RAGGED_MM` either way — not whichever
        // fascicle its corner happens to land on.
        let i = (cy as usize * canvas.size() as usize + cx as usize) * 4;
        let got = canvas.albedo_px.get(i..i + 4).map(|p| [p[0], p[1], p[2], p[3]]).unwrap_or([0; 4]);
        let got_rough = canvas.rough_px.get(i..i + 4).map(|p| [p[0], p[1], p[2], p[3]]).unwrap_or([0; 4]);
        let lo = ((depth - RAGGED_MM - 0.1).max(0.0) * 10.0) as u32;
        let hi = ((depth + RAGGED_MM + 0.1) * 10.0) as u32;
        let matches = (lo..=hi).any(|tenth| {
            let (want, want_rough) = texel_mean(&limb(), tenth as f32 / 10.0, s.tile_mm, s.seed);
            got == [enc(want[0]), enc(want[1]), enc(want[2]), 255] && got_rough[1] == enc(want_rough)
        });
        assert!(matches, "the centre {got:?} is not the grain-averaged tissue near {depth} mm");
        assert_eq!(
            [got_rough[0], got_rough[2], got_rough[3]],
            [enc(depth / limb().span_mm()), 0, 255],
            "the rough pixel is not (true depth fraction, _, dielectric, peeled)"
        );

        // A texel the stamp never reached keeps the intact surface exactly — and reports no wound.
        assert_eq!(canvas.depth_at(0, 0), Some(0.0));
        assert_eq!(canvas.albedo_px.get(0..4), Some(&canvas.base_rgba[..]));
        assert_eq!(canvas.rough_px.get(0..4), Some(&[0, canvas.base_rough, 0, 0][..]));
        assert_eq!(canvas.normal_px.get(0..4), Some(&[128, 128, 255, 255][..]), "intact skin is flat");
    }

    /// **The crater's walls face its centre.** Left of the centre the normal leans `+X`, right of it
    /// `-X`; above it (row 0 is the top of the image, glTF's `+Y`) it leans `-Y`, below it `+Y`.
    #[test]
    fn the_crater_walls_lean_toward_its_centre() {
        let (_images, mut canvas) = scratch(64);
        canvas.paint_uv(Vec2::new(0.5, 0.5), 0.25, 20.0, 0);
        canvas.shade(&FlaySettings::default());
        let (cx, cy) = centre(&canvas);
        let px = |x: u32, y: u32| {
            let i = (y as usize * canvas.size() as usize + x as usize) * 4;
            canvas.normal_px.get(i..i + 4).map(|p| [p[0], p[1], p[2]]).unwrap_or([0, 0, 0])
        };
        assert!(px(cx - 8, cy)[0] > 128, "the left wall must lean right: {:?}", px(cx - 8, cy));
        assert!(px(cx + 8, cy)[0] < 128, "the right wall must lean left: {:?}", px(cx + 8, cy));
        assert!(px(cx, cy - 8)[1] < 128, "the top wall must lean down the image: {:?}", px(cx, cy - 8));
        assert!(px(cx, cy + 8)[1] > 128, "the bottom wall must lean up the image: {:?}", px(cx, cy + 8));
        assert_eq!(px(cx, cy)[..2], [128, 128], "the floor of a symmetric crater is level");
    }

    /// **The two data channels are the depth buffer, not a look.** A shader reading R gets
    /// millimetres back; a shader reading A gets the wound's own mask, which is the thing it cannot
    /// infer from the albedo — flayed muscle and a red shirt are the same red.
    #[test]
    fn the_rough_channels_report_the_depth_and_the_wound() {
        let (_images, mut canvas) = scratch(32);
        let s = FlaySettings::default();
        let layers = limb();
        // Into the fat: past the 1.9 mm skin, short of the 9.1 mm muscle start.
        canvas.paint_uv(Vec2::new(0.5, 0.5), 0.2, 5.0, 0);
        canvas.shade(&s);

        let (cx, cy) = centre(&canvas);
        let depth = canvas.depth_at(cx, cy).unwrap_or(0.0);
        assert_eq!(layers.at(depth).0, Layer::Fat, "the fixture is not in the fat at {depth} mm");

        let i = (cy as usize * canvas.size() as usize + cx as usize) * 4;
        let px = canvas.rough_px.get(i..i + 4).unwrap_or(&[]);
        let want_r = (255.0 * (depth / layers.span_mm()).min(1.0)).round() as u8;
        assert_eq!(px.first().copied(), Some(want_r), "R is not the depth fraction");
        assert_eq!(px.get(3).copied(), Some(255), "a peeled texel is not marked peeled");
        // The claim R actually makes: the byte reads back as the millimetres that were removed.
        let read_back = px.first().copied().unwrap_or(0) as f32 / 255.0 * layers.span_mm();
        assert!(
            (read_back - depth).abs() < layers.span_mm() / 255.0,
            "R read back as {read_back} mm, not {depth} mm"
        );

        let untouched = canvas.rough_px.get(0..4).unwrap_or(&[]);
        assert_eq!(untouched.first().copied(), Some(0), "intact skin reports a depth");
        assert_eq!(untouched.get(3).copied(), Some(0), "intact skin reports a wound");
    }

    /// The crater is what the smooth falloff is for: deepest at the centre, shallower outward, and
    /// every band on show at once.
    #[test]
    fn a_crater_shows_every_band_at_once() {
        let (_images, mut canvas) = scratch(64);
        for hit in 0..12 {
            canvas.paint_uv(Vec2::new(0.5, 0.5), 0.25, 4.0, hit);
        }
        for layer in Layer::ALL {
            assert!(
                canvas.exposed_area(layer) > 0,
                "a crater dug to marrow must show {layer:?} somewhere on its rim"
            );
        }
        let (cx, cy) = centre(&canvas);
        let deep = canvas.depth_at(cx, cy).unwrap_or(0.0);
        let rim = canvas.depth_at(cx + 14, cy).unwrap_or(f32::INFINITY);
        assert!(deep > rim, "the centre must be deeper than the rim, {deep} vs {rim}");
    }

    #[test]
    fn flush_uploads_only_when_dirty() {
        let (mut images, mut canvas) = scratch(16);
        assert!(!canvas.flush(&mut images), "a fresh canvas has nothing to upload");
        canvas.paint_uv(Vec2::new(0.5, 0.5), 0.2, 5.0, 3);
        assert_eq!(canvas.dirty_since(), Some(3));
        canvas.shade(&FlaySettings::default());
        assert!(canvas.flush(&mut images));
        assert!(!canvas.is_dirty());
        assert!(!canvas.flush(&mut images), "a clean canvas costs no bandwidth");
    }

    /// **The frozen digest.** A scripted walk of hits, run as a value.
    ///
    /// Pinned rather than merely compared to itself: a refactor that changed the falloff, the integer
    /// rounding or the saturation would still pass a self-consistency test and would silently move
    /// every consumer's golden. Moving this number is a deliberate act.
    ///
    /// **Re-blessed once, in 0.8.8 (AG-153), `0x46cc_108d_9766_f0be` to `0xecf0_7283_e699_8576`, on purpose**:
    /// the crater's falloff is no longer a smoothstep over the whole radius (`profile`: a wall over the outer
    /// `WALL` of it and a pit, `PIT`) and a crater of eight texels' radius or more has a rim out of round
    /// (`rim_amplitude`). That is the model change `docs/research/wound-margin.md` is about: a wound has an edge.
    #[test]
    fn the_scripted_wound_is_frozen() {
        let (_images, mut canvas) = scratch(64);
        let script: [(f32, f32, f32, f32); 6] = [
            (0.50, 0.50, 0.20, 3.0),
            (0.52, 0.48, 0.10, 6.0),
            (0.50, 0.50, 0.20, 9.0),
            (0.30, 0.70, 0.30, 2.0),
            (0.50, 0.51, 0.05, 20.0),
            (0.70, 0.30, 0.15, 12.0),
        ];
        for (i, &(u, v, r, d)) in script.iter().enumerate() {
            canvas.paint_uv(Vec2::new(u, v), r, d, i as u32);
        }
        assert_eq!(canvas.digest(), 0xecf0_7283_e699_8576);
    }

    /// **A crater has a wall and a pit** (`profile`): nothing at the rim, all of it at the centre, monotone between, and by half the radius
    /// most of the way down. The bowl of 0.8.7 was half of it there, which is what spent a third of the radius on skin and fat.
    #[test]
    fn a_crater_has_a_wall_and_a_pit() {
        assert_eq!(profile(0.0), 0.0);
        assert!((profile(1.0) - 1.0).abs() < 1.0e-6);
        let mut last = 0.0;
        for i in 0..=100 {
            let w = profile(i as f32 / 100.0);
            assert!(w >= last - 1.0e-6, "the profile fell at {i} per cent");
            last = w;
        }
        assert!(profile(0.5) > 0.8, "half way in the crater is {} of its depth: a bowl, not a wall", profile(0.5));
        // The pit: the floor keeps deepening to the centre, so a stack of hits is not a bore.
        assert!(profile(1.0) - profile(0.6) > 0.05, "the floor is a plateau: {} to {}", profile(0.6), profile(1.0));
    }

    /// **A big crater's rim is out of round, a small one is round to the texel** (`rim_amplitude`), and the same hit in the same place tears the same.
    #[test]
    fn a_big_crater_is_out_of_round_and_a_small_one_is_not() {
        let spread = |r_tex: f32| {
            let (_i, mut canvas) = scratch(256);
            canvas.paint_uv(Vec2::new(0.5, 0.5), r_tex / 256.0, 20.0, 0);
            let far = reach(&canvas, r_tex, 24);
            far.iter().cloned().fold(0.0f32, f32::max) - far.iter().cloned().fold(f32::MAX, f32::min)
        };
        let (big, small) = (spread(48.0), spread(10.0));
        assert!(big > 0.06, "a crater of 48 texels' radius is round to {big} of its radius");
        assert!(small < 0.2, "a crater of 10 texels' radius is out of round by {small} of its radius: a texel is a tenth of it");
        let again = {
            let (_i, mut canvas) = scratch(256);
            canvas.paint_uv(Vec2::new(0.5, 0.5), 48.0 / 256.0, 20.0, 0);
            let far = reach(&canvas, 48.0, 24);
            far.iter().cloned().fold(0.0f32, f32::max) - far.iter().cloned().fold(f32::MAX, f32::min)
        };
        assert_eq!(big, again, "the same hit in the same place tore differently");
    }

    /// How much a canvas's normals vary over the texels peeled to between `lo` and `hi` millimetres: the standard deviation of the
    /// `x` and `y` bytes, together.
    fn normal_spread(canvas: &FlayCanvas, lo: f32, hi: f32) -> f32 {
        let (mut n, mut sum, mut sum2) = (0.0f32, 0.0f32, 0.0f32);
        for i in 0..canvas.depth.len() {
            let d = canvas.depth[i] as f32 / HMM_PER_MM;
            if d < lo || d > hi {
                continue;
            }
            for c in 0..2 {
                let v = canvas.normal_px[i * 4 + c] as f32;
                sum += v;
                sum2 += v * v;
                n += 1.0;
            }
        }
        if n == 0.0 { 0.0 } else { (sum2 / n - (sum / n) * (sum / n)).max(0.0).sqrt() }
    }

    /// **A wound's floor is lumpy, its lip is pale, and what is not a wound is untouched.** On the 256 canvas (3.9 mm a texel) a crater to the
    /// muscle has a floor whose normals vary (lumps: nodules and crevices), and the same crater on a 64 canvas (15.6 mm a texel, a lump is
    /// under a texel) has none; bone, where the depth saturates, is smooth; the first millimetre is a lip paler than the skin round it; and a
    /// texel at depth zero is exactly the intact surface.
    #[test]
    fn a_wound_has_a_lumpy_floor_a_pale_lip_and_leaves_the_skin_alone() {
        let s = FlaySettings::default();
        let lumpy = |size: u32| {
            let (_i, mut canvas) = scratch(size);
            canvas.paint_uv(Vec2::new(0.5, 0.5), 0.2, 16.0, 0);
            canvas.shade(&s);
            canvas
        };
        let (fine, coarse) = (lumpy(256), lumpy(64));
        // The floor only: past the wall, where the depth is within a tenth of the crater's 16 mm.
        let (a, b) = (normal_spread(&fine, 14.5, 40.0), normal_spread(&coarse, 14.5, 40.0));
        assert!(a > 4.0, "the floor of a crater on a 256 canvas has normals that vary by {a} bytes: it is smooth");
        assert!(b < 0.25 * a, "a 15.6 mm texel cannot hold a 16 mm lump, and the floor varies by {b} against {a}");
        // The skin round it, and the lump store, are untouched where nothing is peeled.
        let flat = [128, 128, 255, 255];
        for (i, &d) in fine.depth.iter().enumerate() {
            if d == 0 {
                assert_eq!(&fine.albedo_px[i * 4..i * 4 + 4], &fine.base_rgba, "an intact texel was recoloured");
                assert_eq!(fine.lump[i], 0, "an intact texel is lumpy");
            }
        }
        assert!(fine.normal_px.chunks(4).any(|n| n == flat));
        // Bone is smooth: a stack of hits that saturates the depth is a plateau, and a plateau has no lumps to light.
        let (_i, mut deep) = scratch(256);
        for t in 0..4 {
            deep.paint_uv(Vec2::new(0.5, 0.5), 0.2, 60.0, t);
        }
        deep.shade(&s);
        let centre = ((128 * 256 + 128) * 4) as usize;
        assert_eq!(&deep.normal_px[centre..centre + 4], &flat, "the bone at the bottom of the crater is lumpy");
        // The lip: the outermost peeled texels, on the whole, are paler than the intact skin. The mean and not the brightest, because the
        // torn lookup can put a texel of cream fat at the rim whatever the lip does.
        let luma = |p: &[u8]| 0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32;
        let base = luma(&fine.base_rgba);
        let rim: Vec<f32> = (0..fine.depth.len())
            .filter(|&i| fine.depth[i] > 0 && (fine.depth[i] as f32 / HMM_PER_MM) < 0.8)
            .map(|i| luma(&fine.albedo_px[i * 4..i * 4 + 4]))
            .collect();
        assert!(!rim.is_empty(), "no texel of the rim is under 0.8 mm deep");
        let pale = rim.iter().sum::<f32>() / rim.len() as f32;
        assert!(pale > base + 6.0, "the lip's mean is {pale} against the skin's {base}: it is not paler");
    }

    /// How far from the middle of a `size` canvas each of `sectors` directions is peeled at all, in crater radii.
    fn reach(canvas: &FlayCanvas, r_tex: f32, sectors: usize) -> Vec<f32> {
        let size = canvas.size();
        let c = size as f32 / 2.0;
        let mut far = vec![0.0f32; sectors];
        for y in 0..size {
            for x in 0..size {
                if canvas.depth_at(x, y).unwrap_or(0.0) > 0.0 {
                    let (dx, dy) = (x as f32 + 0.5 - c, y as f32 + 0.5 - c);
                    let turn = dy.atan2(dx) / core::f32::consts::TAU;
                    let s = (((turn + 1.0).fract()) * sectors as f32) as usize % sectors;
                    far[s] = far[s].max((dx * dx + dy * dy).sqrt() / r_tex);
                }
            }
        }
        far
    }

    /// **An entrance is round and an exit is a star.** The same crater, plain and torn: the plain one is as far from its
    /// middle in every direction to within a texel, the torn one runs out in five to seven splits, each well past the rim,
    /// between which the margin is still near the rim. Glessner Lee's two chest plates, as numbers.
    #[test]
    fn a_torn_crater_is_a_star_and_a_plain_one_is_round() {
        let r_tex = 0.08 * 128.0;
        let (_i, mut plain) = scratch(128);
        plain.paint_uv(Vec2::new(0.5, 0.5), 0.08, 9.0, 0);
        let (_i, mut torn) = scratch(128);
        torn.paint_uv_torn(Vec2::new(0.5, 0.5), 0.08, 9.0, 1.0, 0);
        let round = reach(&plain, r_tex, 36);
        let (lo, hi) = (round.iter().copied().fold(f32::MAX, f32::min), round.iter().copied().fold(0.0, f32::max));
        assert!(hi - lo < 0.3, "a plain crater is not round: it reaches {lo}..{hi} radii");
        let star = reach(&torn, r_tex, 36);
        let (lo, hi) = (star.iter().copied().fold(f32::MAX, f32::min), star.iter().copied().fold(0.0, f32::max));
        assert!(hi >= 1.7, "no split of a torn crater runs past 1.7 radii: the farthest is {hi}");
        assert!(lo >= 0.55, "a torn crater's margin falls to {lo} radii between splits, which is a bite out of it");
        // Splits: runs of directions that reach well out.
        let out: Vec<bool> = star.iter().map(|&f| f > 1.25).collect();
        let splits = (0..out.len()).filter(|&i| out[i] && !out[(i + out.len() - 1) % out.len()]).count();
        assert!((3..=7).contains(&splits), "a torn crater has {splits} splits that reach 1.25 radii, not three to seven: {star:?}");
    }

    /// **A split opens the skin; it does not bore.** Past the rim no texel of a torn crater is deeper than
    /// [`TEAR_SPLIT_DEPTH`] of the crater's own depth, and the hub is as deep as a plain crater's.
    #[test]
    fn a_split_is_shallower_than_the_crater_it_runs_from() {
        let (_i, mut plain) = scratch(128);
        plain.paint_uv(Vec2::new(0.5, 0.5), 0.08, 9.0, 0);
        let (_i, mut torn) = scratch(128);
        torn.paint_uv_torn(Vec2::new(0.5, 0.5), 0.08, 9.0, 1.0, 0);
        let (cx, cy) = centre(&torn);
        assert_eq!(torn.depth_at(cx, cy), plain.depth_at(cx, cy), "the hub of a torn crater is not as deep as a plain one's");
        let r_tex = 0.08 * 128.0;
        let mut deepest_out = 0.0f32;
        for y in 0..128 {
            for x in 0..128 {
                let (dx, dy) = (x as f32 + 0.5 - 64.0, y as f32 + 0.5 - 64.0);
                if (dx * dx + dy * dy).sqrt() > 1.3 * r_tex {
                    deepest_out = deepest_out.max(torn.depth_at(x, y).unwrap_or(0.0));
                }
            }
        }
        assert!(deepest_out > 0.0, "no split reaches past 1.3 radii");
        assert!(deepest_out <= TEAR_SPLIT_DEPTH * 9.0 + 0.05, "a split is {deepest_out} mm deep, deeper than {TEAR_SPLIT_DEPTH} of 9 mm");
    }

    /// **Tearing is a function of the place, and zero is no tearing at all.** The same hit twice is the same bytes; a hit a
    /// texel over tears differently; and `tear` of zero, or of NaN, is `paint_uv` to the byte, which is what keeps every entrance wound
    /// and every recorder that has no exit exactly what it was.
    #[test]
    fn a_tear_is_a_pure_function_of_the_place_and_zero_is_none() {
        let at = |u: f32, tear: f32| {
            let (_i, mut c) = scratch(128);
            c.paint_uv_torn(Vec2::new(u, 0.5), 0.08, 9.0, tear, 0);
            c.digest()
        };
        assert_eq!(at(0.5, 1.0), at(0.5, 1.0), "the same hit tore two ways");
        assert_ne!(at(0.5, 1.0), at(0.5 + 1.0 / 128.0, 1.0), "two hits a texel apart tore the same way");
        let (_i, mut plain) = scratch(128);
        plain.paint_uv(Vec2::new(0.5, 0.5), 0.08, 9.0, 0);
        assert_eq!(at(0.5, 0.0), plain.digest(), "a tear of zero is not paint_uv");
        assert_eq!(at(0.5, f32::NAN), plain.digest(), "a tear of NaN is not no tear");
        assert_ne!(at(0.5, 0.5), at(0.5, 1.0), "half torn and fully torn are one crater");
    }

    /// A torn crater at the corner of the canvas stays on it: the splits that run off the edge are cut, nothing is indexed past it.
    #[test]
    fn a_torn_crater_at_a_corner_stays_on_the_canvas() {
        let (_i, mut c) = scratch(64);
        c.paint_uv_torn(Vec2::new(0.0, 0.0), 0.2, 9.0, 1.0, 0);
        c.paint_uv_torn(Vec2::new(1.0, 1.0), 0.2, 9.0, 1.0, 1);
        assert!(c.depth_at(0, 0).unwrap_or(0.0) > 0.0 && c.depth_at(63, 63).unwrap_or(0.0) > 0.0, "a crater in a corner peeled nothing");
    }
}
