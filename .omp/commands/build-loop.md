---
description: One iteration of the bevy_carnage improvement loop (gore that looks like the horror films it learns from, and stays deterministic)
argument-hint: "[optional focus, e.g. 'arterial spray only' or 'issue #3 only']"
---

One run of this command is one iteration of a long-running loop on bevy_carnage, a Bevy 0.20 crate of deterministic runtime gore: it plane-cuts a character's own meshes into watertight-capped chunks and breaks a skinned body where its skeleton is, bores bullet channels, peels craters (`flaymap`), opens lacerations, spills viscera, and drives blood off the wounds that result: spatter by bloodstain-pattern physics (`bloodstain`), runs, pools and drying on CPU-owned canvases (`wetmap`), and the `flesh` material and `preset` (`GorePlugin`, `Gore`, `GoreHit`) that put all of it on a loaded glTF character in two lines. Games consume it for violence a player reads at a glance: Sanctuary (decision 0241 made this repository the source of truth) and bevy_dark_factory's family integration F12, destructible props and damage dials (`gh issue view 108 --repo dark-incorporated/bevy_dark_factory`). Do ONE improvement per iteration, the single most valuable one from a game developer's point of view, finish it completely, then stop.

This crate must be reproducible AND convincing AND cheap on the caller's frame. A digest that changes between two runs of the same build, a frozen golden that moves, blood that reads as red paint, plastic or a decal sticker, and a hit that spikes the frame are all failures, however good one screenshot looks. **The owner's direction (2026-10-30): this loop learns its look from horror films.** The judgement on anything visible is a practical-effects artist's on a horror set (Rob Bottin on *The Thing*, Tom Savini, the KNB EFX shop, the *Evil Dead* and *Terrifier* effects crews): does the blood have the right red for its age, the right body as it beads and runs, does a wound have an edge, a depth and a wet sheen, does a pool spread and crust as one does, does debris read as meat and bone. Under that eye sits a forensic scientist's: the spatter is the ballistics `bloodstain` models and the drying is the rheology it cites. The look is replicated **procedurally** (shaders, particles, canvases, meshes, parameters), never by shipping someone's photograph.

You do all the work yourself, including looking at any images and judging them. Do not delegate
visual review.

Extra instructions for this run (a focus such as "arterial spray only"): $ARGUMENTS
They override the choice of item in section 4, not the gates.

State lives in `docs/loop-log.md`; rules and traps live in `CLAUDE.md`. Follow them; do not
re-derive them.

The driver times every step of a pass from the omp session afterwards (`~/.local/share/wiggum/steps.py`, rows on the `loop-stats` branch as `steps/<clone>.tsv` and `.md`), so the owner can see which steps take longest and cut them. The phase a step belongs to comes from you: **run `python3 ~/.local/share/wiggum/steps.py mark <phase>` as you enter each phase**, with one of `orient`, `bugs`, `choose`, `research`, `build`, `prove`, `gate`, `ship`, `release`, `log`. It prints one line and costs nothing. Without the marks every step lands in `orient`.

## Every pass ships: commit, push, publish, docs, demos (owner, 2026-10-31)

The owner's rule for every pass of this loop. It is checked: after each pass the driver audits it from git (`tools/loop/steps.py audit` in bevy_dark_factory) and records commits on `main`, commits left unpushed, package files changed, release tags, doc and demo files, and what was **missing**, on this repo's `loop-stats` branch.

1. **Commit and push.** Everything the pass made is committed and pushed to `main` (through the gate) before the pass ends. Nothing stays only in the clone.
2. **Publish.** A pass that changed what the package ships releases it in the same pass (the "Release" block in section 8). There is no time limit between releases. A publish crates.io refuses with `429` is logged once and not retried in this pass; the next pass releases, and its audit says so.
3. **Docs.** A change to behaviour or API updates its documentation in the same pass: the README line, the `docs/guide/` section, rustdoc and the CHANGELOG, following https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md. Images and links in a README are absolute URLs.
4. **Demos.** A change anyone can see ships its demo in the same pass: a GIF or still in `docs/` recorded from the crate's own capture path (the recorder examples, `magick` and `ffmpeg`), shown in the README or `docs/`, and listed in `docs/media/README.md`. A media file a published README names is never moved.
5. **Read the last audit first.** In Orient, run `git fetch -q origin loop-stats && git show origin/loop-stats:steps/bevy_carnage_loop-audit.tsv | tail -3`. Anything the previous pass left in `missing` (`push`, `release`, `docs`, `demo?`) is this pass's first job, ahead of its item, because it finishes work already done; only a broken gate comes before it. `demo?` is met by a demo, or by one line in the log saying why the change is not visible.

## Your work is saved even if you are killed — read this before you start

**The driver saves whatever you leave uncommitted, without touching your branch.** Every ten minutes
it records the whole tree to a snapshot ref (`refs/wiggum/pass-N-...`), and again when the pass ends.
Your `git status`, your index and your branch are unchanged — so this is not a substitute for your own
commit step, which is still the only thing that puts work on `main`.
Recover a killed pass's tree with `git -C <repo> log --all --oneline --grep='WIP(pass'` then
`git show <ref>`.

1. **Commit as you go, not once at the end.** Your commit step is last, behind implement, gate and
   benchmark, so a pass killed at the ceiling never reaches it. Commit the research note alone before
   writing code, as your sections already say; then commit the implementation, then the docs and the
   log. Each commit that lands is work a killed pass cannot lose.
2. **A snapshot ref is not a commit on your branch.** `git log` will not show it. Do not look for
   one; if an earlier pass was killed, its tree is in `refs/wiggum/` and nobody's gate ran on it.
3. **Parked work is a branch, and you look for it before you pick an item.** When a pass is stopped or
   killed with its tree uncommitted, the driver's next start commits that tree to a branch
   (`wiggum/parked-pass-N-<time>`, or on the pass's own `loop/iter-N`) in a commit titled
   `WIP(pass N, parked)` and puts the clone back on `main`: you start clean and nobody tells you. So run
   `git branch --list 'wiggum/parked-*'` and `git log --all --oneline -n 5 --grep='parked'` once, after
   you have triaged the filed issues and before you commit to an item. If the newest parked tree is for
   the item you would pick, or for an issue or bug that outranks your pick, **adopt it**: it is a first
   draft that was never built, tested or gated and was cut off mid-work, so read it, bring its files into
   your own `loop/iter-N` branch (`git checkout <parked-branch> -- <paths>` or `git cherry-pick`),
   finish it, and run the real gate on the result before it ships under your name. Never merge the
   parked branch itself and never delete it. If it is stale (its files conflict with `main`, or the item
   was fixed meanwhile), say so in one line in the log and ignore it. A parked branch that exists only on
   this clone holds a lot of data: the driver keeps anything over 10 MB of new data off origin.


## 1. Orient

1. `CLAUDE.md`: the reproducibility non-negotiable, the boundary ("Where the boundary falls"), the
   modules' own rules (goldens are locks, the CPU is the authority, ticks not seconds, seeds are places).
2. `docs/loop-log.md` (create it if missing): what earlier iterations did, found and deferred, and the
   newest iteration's ranked backlog. The next iteration number is the last one plus one.
   `BACKLOG.md` and `BACKLOG_ARCHIVE.md` are the pre-loop ticket history (`AG-001` … `AG-139`); read
   them for what was already tried, never rewrite them.
3. `git log` since the last logged iteration.
4. The `docs/research/<system>.md` note of any system you may touch, and `docs/research/corpus.md` for
   the reading list per domain (create both the first time a pass researches). The older per-module
   notes (`docs/bloodstain.md`, `docs/bloodstain-citations.md`, `docs/wetmap.md`, `docs/flaymap.md`,
   `docs/laceration.md`, `docs/viscera.md`, `docs/cross_section.md`, `docs/fracture_modes.md`,
   `docs/preset.md`, `docs/research-brief.md`, `docs/visual-pass-round-*.md`) are prior research: cite
   them, do not duplicate them.
5. `git status --short`. The driver (`wiggum.sh`) starts a pass only on `main` with no
   tracked changes. If you find tracked changes you did not make, stop and log them; do not commit or
   revert them.
6. **Re-scan the corpus for new sources.** Run `catalog_recent` (limit 200) and add any blood
   rheology, haemodynamics, droplet ballistics and impact, bloodstain pattern analysis, wound and skin
   biomechanics, tissue optics and subsurface scattering, fracture mechanics and brittle or ductile
   fracture simulation, real-time gore and dismemberment rendering, decal and texture-space damage, or
   practical makeup-effects document that `docs/research/corpus.md` does not list, as its own commit
   (`Research: corpus`).

Preflight home-still now, so research is not blocked later:

- `system_status` (MCP) or `hs status`. HomeStill's servers run on other machines; never restart them
  from here. If distill is down, skip `distill_search` and `distill_index`, search with the CLI
  (`hs distill search "<query>" --output json`) or `paper_search`, and list each unindexed note under
  `Pending index`. Record the live state in the log; do not copy an earlier iteration's outage note.
- Scribe: check `scribe_health`. `scribe_convert` may queue or fail; then rely on the corpus.
- **An outage is not a reason to idle.** `paper_search`, the CLI and the papers already quoted in this
  repository need no GPU; research from them and build.

## 2. Capture the current state

Build first: `cargo build --release --examples` (default features, `serde` + `flesh`). **Not
`--all-features`:** the `vfx` feature cannot build until `bevy_hanabi` publishes for Bevy 0.20, so the six
`vfx` examples (`carnage`, `capture_carnage`, `pooling`, `capture_pooling`, `ribbons`, `capture_ribbons`)
are skipped by their `required-features`. The supervisor sets `CARGO_TARGET_DIR`, so the binaries are
under `$CARGO_TARGET_DIR/release/examples/`, never `./target`. Keep everything under `/tmp/cN/` (`N` the
iteration), the baseline under `/tmp/cN/before/`. Keep a worktree of the starting commit for later
comparisons, **detached, at a commit, and off `/tmp`**: `git worktree add --detach ~/cN_base <sha>`, built
with `CARGO_TARGET_DIR=~/cN_base-target`.

Section 9 says how to name, build and remove that baseline. A baseline left behind costs the next pass
a build cache's worth of disk, and one with `main` checked out stops anything else checking `main` out.

**The numbers are the state.** This crate's cost lands on its caller's frame and its output must be
reproducible, so measure both the way a caller pays for them:

- **Cost:** `$CARGO_TARGET_DIR/release/examples/bench_carnage` (sixteen scripted bodies, 600 ticks — `const TICKS: u32 = 600` in the example, no
  window, no GPU). Record its `METRIC carnage_frame_ms=`, `METRIC carnage_total_ms=` and
  `METRIC carnage_p99_ms=` lines and its `digest` line in `/tmp/cN/before/bench.txt`. It refuses to
  report a timing when the carnage moved or the fastest reps disagree ("NO STABLE PLATEAU"): that refusal
  is a reading, not a number to retry around; wait and re-run once, then log it.
- **Reproducibility:** the headless recorders print one digest each. Run
  `capture_preset --out /tmp/cN/before/preset` (`preset: frames= ticks= digest=`) and `capture_skinned`
  in each variant the item touches (`--break`, `--shirt`, `--through`, `--guard`, `--stroll`, `--blows`,
  `--quiet`; `skinned: frames= ticks= digest=`), each **twice**; the two digests of a variant must be equal.
  The terminal examples that print model digests (`laceration_curve`, `rod_determinism`, `canvas_digest`,
  `flay_digest`, `dry_timeline`, `rheology`, `stain_sweep`, `pattern_classes`, `fragment_energy`,
  `fracture_modes_bar`) are run for the module you touch.
- **Everything at once:** `FONT=/System/Library/Fonts/Supplemental/Arial.ttf QA_BIN=$CARGO_TARGET_DIR/release/examples tools/qa.sh /tmp/cN/before/qa [name ...]`
  renders every recorder, terminal and autopiloted windowed example (off-screen, so a covered window or a
  locked screen does not blacken it) and writes `warnings.txt`, `digests.txt`, `missing.txt` and contact
  sheets. Name only the examples the item touches; a full run is long.

The loop's host shares its cores with other loops, so a single run is noise: run the bench three times
before and three times after, back to back on the same machine, and compare medians of medians.

**The visible state is captured beside its references.** This host (big_mac) has a native Metal display
and GPU, so the recorders render real frames. When the item touches anything visible (blood, wounds,
gore, fracture debris, splatter, pooling, drips, stains, the flesh material), do this **before you look
at your own frames**:

1. **Gather the reference set from horror films.** `web_search` for practical-effects gore stills,
   blood splatter on walls, wound makeup close-ups, dismemberment and body horror, by film and by artist:
   *The Thing* (1982), *Evil Dead* (1981 and 2013), *Hellraiser*, *Re-Animator*, *Terrifier*, *The Fly*
   (1986), *Saw*, and FX-artist interviews and behind-the-scenes pages (KNB EFX, Tom Savini, Rob Bottin,
   the *Terrifier* crew). Then the golem MCP's `search_reference_images` (Openverse, Wikimedia Commons,
   Pexels, Brave; a numbered contact sheet with each item's source and licence). Look at each candidate
   with `read` on its image URL.
2. **Keep 4-6 references that show the item's phenomenon**, saved under `/tmp/cN/refs/` (`curl -L -o`),
   never in the repository. For each, write down its URL, the film (or source) and its licence.
3. **Capture the crate's matching shot**: the recorder or autopiloted example that shows the same thing,
   framed the same way where the example's `--eye x,y,z` / `--at x,y,z` allows (`capture_skinned`), at
   the moment the reference shows (fresh, mid-run, dried). Keep it under `/tmp/cN/before/`.

## 3. Inspect closely (you judge the results yourself)

Read every number and every image yourself. Write the **gore checklist** to `/tmp/cN/findings.md`,
each line with a measurement, not an impression:

- **reproducibility:** each recorder variant's digest equal across its two runs, and equal to the
  baseline's unless this item means to move it (quote both hex digests);
- **the locks:** `hash_f32_is_frozen`, `the_spatter_model_is_frozen`, `the_stain_placement_is_frozen`,
  `the_pool_model_is_frozen`, `the_bruise_model_is_frozen`, `the_burn_model_is_frozen`,
  `the_wick_model_is_frozen`, `the_flesh_tables_are_frozen`, `the_tear_is_frozen`,
  `the_strips_are_frozen`, `the_scripted_wound_is_frozen` and `the_measured_rows_are_the_papers` pass
  untouched;
- **mass:** a wetmap's drip and spread passes conserve every byte (`tests/wetmap.rs`); quote the total
  before and after a scripted run, and the loss on saturated crust if any;
- **geometry:** a fracture's pieces closed, manifold and consistently wound by `isomesh` (count of
  fragments failing each, from the fracture examples or tests), and a skinned break's piece count;
- **cost:** the bench's `carnage_frame_ms` and `carnage_p99_ms`, median of three, before and after;
- **the reference checklist**, one line per observation in the research note's `## Reference set`, each
  measured off the capture and quoted against the same measure off the reference. At least:
  - **colour by age:** fresh blood's sRGB sampled from the capture
    (`magick f.png -crop WxH+X+Y -resize 1x1\! -format '%[pixel:p{0,0}]' info:`) against fresh blood in the
    reference, and again on a dried stain or crust (bright arterial red to brown-black);
  - **viscosity:** a run's width, how far it travels before it arrests, and whether it ends in a bead,
    in texels or millimetres at the canvas's `uv_per_metre`, against a reference drip;
  - **spray:** stain count, the size spread (smallest to largest) and the elongation of stains on a wall
    against their impact angle (`capture_skinned --through`), against a splatter reference;
  - **pool:** radius over time, the edge (crisp rim, darkened ring as it dries) and the highlight's
    break-up, against a pooling reference;
  - **wound:** the edge (a lip, a ragged margin), the depth reading (crater walls darker than the floor)
    and the wet sheen (a specular highlight on the film), against a wound-makeup close-up;
  - **fracture and debris:** a cut face reads as wet muscle, fat and bone rather than a flat cap, and the
    pieces' size spread, against a dismemberment or body-horror still;
- **the frames:** each changed capture read at full frame and zoomed, beside its reference.

## Owner input: recommend, record, proceed

**This loop runs fully automated (owner, 2026-10-29): never stop to wait for the owner.** When a pass needs the owner's input (a design choice the issue or item does not settle, such as a colour, a coverage, an API shape, a schema change or which of two behaviours; or a protected file that must change), it answers the question itself:

1. **Recommend.** Name the two or three real options and pick one, with one line of why. Prefer the conservative choice: reversible, conventional, consistent with the existing code and the issue's stated intent, and the smallest that meets the acceptance.
2. **Assume the owner chose it, and build it this pass.** A decision is never a reason to skip an item or end a pass.
3. **Record it where the owner reviews.** On the issue, a comment beginning `Assumed decision (owner may override):` with the options, the pick and why, and the label `assumed-decision` (`gh label create assumed-decision --color FBCA04 2>/dev/null; gh issue edit <N> --add-label assumed-decision`). With no issue, in the commit body and the log line, beginning `Assumed decision:`.
4. **The owner's reply wins.** An owner comment that contradicts an assumed decision overrides it: the next pass that reads it reworks the item to the owner's choice before anything else.
5. **Earlier blocks are lifted.** An issue or backlog item logged as blocked on, or waiting for, an owner decision is workable now: decide it this way.
6. **Protected checks change only additively.** An assumed decision may add to a protected test or check (a new row or entry, a new field's empty default in a literal, a count or version raised by exactly what the change adds), with the commit message `assumed owner exception: <what>`. It never removes, loosens, skips or re-pins an existing assertion, golden, eval answer or threshold; that is the one block left, so say so on the issue and go on.
7. **Never, whatever you assume.** These need the owner's word in a conversation, and no prompt can give it: yanking a release or changing crate ownership; force-pushing, rewriting history or deleting branches; deleting files or work this pass did not create; editing this command file; spending money or sharing credentials.

## Filed issues come before new features

**A filed issue is a consumer telling you what broke, which outranks anything this pass invented.**
Before you pick an item, list the repository's open issues and triage them:

```sh
gh issue list --repo dark-incorporated/bevy_carnage --state open --limit 100 \
  --json number,title,labels,createdAt --jq '.[] | "#\(.number) \(.title)"'
```

(If `gh` is absent, the same list is
`https://api.github.com/repos/Ladvien/bevy_carnage/issues?state=open&per_page=100`.)

Then, in this order:

1. **A bug issue is the pass's item, ahead of any enhancement.** Anything reporting wrong behaviour —
   a crash, a panic, a visual artifact, a value that is implausible, a documented promise that is not
   kept — is a bug. Take the oldest such issue that is still reproducible, and say why it beats the
   others.
2. **An issue labelled `consumer` is the pass's item next, ahead of the series and your own candidates.**
   A project built on this crate needs it and is waiting. The `consumer:<name>` label says which project;
   `consumer:bevy_dark_factory` is the owner's editor. Its own work resumes when the issue is closed as completed
   and it moves its git pin to a `main` commit containing the fix. Take the oldest one you can work this pass.
   - **Build the general form.** The requester is the example, never a name in this crate's API, a path or a
     default: this crate serves other projects too. Where an issue asks for something shaped to one project,
     build the general form and say so on the issue.
   - **Prove it here**, with a test in this repository that fails before and passes after against the issue's
     acceptance. Never build or run the requesting project to prove it.
   - **Pushing to `main` is the delivery.** Requesters pin a git revision, so a consumer fix never needs a
     publish or a version bump.
   - **One that needs a decision the issue does not settle is decided, not skipped:** recommend, record and
     build it (see "Owner input: recommend, record, proceed"). Only a change that would weaken a protected
     check cannot be worked: say exactly what is needed on the issue (`gh issue comment`), write one log
     line, and go on. A blocked consumer issue never stops the pass.
   - **Read `git log` first.** While this loop is stopped, the requester's loop may fix the issue here at the
     source, in its own commits with tests; check whether that has happened so the work is not done twice.
3. **Every other open enhancement issue is the pass's item next, ahead of your own candidates** (the
   owner's rule, 2026-10-29): a filed request was asked for; a candidate this pass invents was not. The
   order of work is: **bugs first** (a bug issue, then a bug already recorded in the backlog or the log),
   then `consumer` issues, then every other open enhancement issue, oldest first. Only when each of them
   is done, blocked or a release issue does "Pick the single most valuable item" choose a feature of your
   own. A blocked issue gets a `gh issue comment` saying exactly what it needs and is skipped; it never
   stops the pass. Name the issues you skipped, and why, in the log line, so the next pass sees the list
   was worked through.
4. **Close the loop.** An issue your pass fixes is referenced in the commit body (`Fixes #N`) **and**
   in the report, so the next pass can see it was addressed. Do not close an issue you did not fix,
   and do not close one whose fix this pass did not verify. For a `consumer` issue the close is the signal
   the requester waits for: close it only through a verified fix pushed to `main`, never as not planned
   without the owner's word.
5. **A release issue is done by the next release.** An issue asking to publish or bump a version is closed by the pass whose release (the "Release" block in the ship section) publishes it, with the crates.io URL.

**When the API fallback answers `403` or `429`** (rate limit — 60 requests/hour per IP unauthenticated,
shared by everything on this network), do not treat it as "no issues": write one line saying the list
could not be read and continue. Never spend a second call to retry it in the same pass, and never infer
from silence that nothing is filed.

**An empty issue list costs nothing:** if it is empty or `gh` is unreachable, write one line saying so
and continue — never spend a pass on the lookup.

## 4. Pick the single most valuable item

**You are here only when no bug and no open issue can be worked this pass** (see "Filed issues come
before new features"); otherwise that issue is the item. Rank candidates from your findings, the loop log's backlog and the extra instructions, always asking
what matters most to a game developer who will ship this:

1. A wrong or unreproducible result: a digest that differs between two runs of one build, a frozen
   golden that moved, a panic or a silently accepted bad input on a caller's mesh, a wetmap pass that
   loses or invents blood, a fracture piece that is open or inside out, a hit that wounds the wrong mesh.
2. A look that does not read next to the horror-film references: blood the wrong red for its age, paint
   or plastic or a decal sticker where a film shows a wet, beading liquid; spray read as dots; a pool with
   no rim and no crust; a wound with no edge, depth or sheen; debris that reads as clean cheese or plastic.
3. A cost the caller cannot afford: a rise in `carnage_frame_ms` or `carnage_p99_ms`, a hit tick that
   spikes, a bake that hitches, memory per dressed character that grows with nothing to show for it.
4. API friction: dressing a character or a prop takes more than the two-line inclusion; a setting a
   game wants to author in RON or change at runtime (a breakable prop, the violence dials) that it cannot;
   a value the caller must recompute because the crate hides it.
5. Missing features a gore-heavy game expects: an arterial spurt in time with a pulse, a blood trail
   behind a wounded mover, a stump that reads as one, stains that age on any surface, soak that spreads
   through cloth, gibs that bleed where they land.

Ownership is a gate: the convex decomposition, naming the part that detaches, when the bake may run,
and everything after a fragment exists (rigid bodies, colliders, launch impulses, despawn) are the
caller's (`CLAUDE.md`, "Where the boundary falls"). This crate never learns what died: no health, no
factions, no damage, no physics, and no name in it says "gun", "figurine" or "unit". Do not build those
here; add a hook instead.

State in one sentence why your pick beats the runner-up. If it is too big, carve off the most valuable
self-contained slice. Record the ranked candidate list in the log, not only the winner.

**A pass never ends idle** (owner, 2026-10-29: "If they're not addressing bugs or enhancing features, they need to evolve their own code base to support Bevy Emerge better or add new features. Research these things in home still, implement them. They shouldn't stay idle."). An empty issue list or a stale backlog is this step's input, never a reason to ship nothing, and a candidate is not "speculation" once it is researched. When the ranked list above gives no workable item, take the first of these that applies:
1. **What the crate's consumers hand-roll.** Read how Sanctuary uses this crate (`gh repo clone Ladvien/sanctuary /tmp/cN/sanctuary -- --depth 1`, then `rg -l bevy_carnage`; removed afterwards) and bevy_dark_factory's family issues (`gh issue list -R dark-incorporated/bevy_dark_factory --label family --state all`, and F12, `gh issue view 108 --repo dark-incorporated/bevy_dark_factory`, destructible props and damage dials). Find a formula, a wrapper or a query either writes by hand around this crate (marking a prop breakable, authoring the violence dials as RON, mapping a hit to `GoreHit`, a policy toggle at runtime), and build its general form here, never named for either. Prove it with a test here.
2. **A feature a shipping game in this domain expects and this crate lacks**, found by research: home-still first (`distill_search`; for a paper the corpus lacks, `paper_search`, `paper_download`, `scribe_convert`, then `distill_index`), then shipped-game practice, then section 5 before any code.
3. **Depth on the weakest existing feature**: measure it against its research note's validation targets and improve the worst gap.

Record the ranked candidates and the pick in the log. A pass that ships nothing names the concrete blocker that stopped all three; "nothing new to do" is not a blocker. **"Not researched yet" is not a blocker either: it is this pass's work.** Take the top item, do section 5 for it now (home-still preflight included), commit the note, and build its first slice. "Building it would be speculation" means the research is missing, so do the research. A log-only iteration is a failed pass, not an outcome, and a pass never copies an earlier iteration's reasons for shipping nothing (measured 2026-10-29: iterations 194 and 195 each shipped a log line saying every item was "not researched", and neither ran the research).

## 5. Research thoroughly (before any code)

Every feature gets researched, new or improved, with no exceptions. Gore here is a physical process
made to look like the films it learns from, so every iteration cites **three** kinds of source:

1. a **physics** source: blood rheology and haemodynamics (shear-thinning, yield stress, clotting,
   drying), droplet ballistics and impact, bloodstain pattern analysis, wound and skin biomechanics,
   tissue optics, fracture mechanics. Search home-still (`distill_search`, then `paper_search`); the
   papers already quoted in `docs/bloodstain-citations.md`, `src/cross_section/layers.rs`,
   `src/laceration/curve.rs` and `src/laceration/tear.rs` (Comiskey, Stam, O'Brien, Bargteil & Hodgins 2002, Müller, Chentanez & Kim 2013) are the
   models in use;
2. a **horror-film reference** source, mandatory for anything visible: the 4-6 references section 2
   gathered (practical-effects stills from *The Thing*, *Evil Dead*, *Hellraiser*, *Re-Animator*,
   *Terrifier*, *The Fly*, *Saw*, and FX-artist interviews and behind-the-scenes pages), each with its URL,
   film and what it shows that the crate must reproduce. A non-visible item (cost, API, determinism) uses
   a shipping game's gore pipeline here instead;
3. a **technique** source: how shipping games render and budget gore (GDC talks and studio write-ups:
   *Dead Space* dismemberment, *Soldier of Fortune*'s GHOUL, *Dead Island 2*'s FLESH, *The Last of Us
   Part II*, *Doom Eternal*), texture-space damage and decals, wet-surface and subsurface shading, and the
   WebGL2 and frame-cost limits the crate keeps.

When the home-still MCP bridge answers `401 Refresh token expired`, search with the CLI,
`hs distill search "<query>" --output json`, which refreshes its own token.

If the corpus lacks one of the three, `paper_search` for it, and record the gap under the note's
`## Pending index`. Write the note with the headings this loop requires (`## Sources`,
`## Craft principle`, `## How shipping games do it`, `## Model adopted`, `## Parameters`,
`## Validation targets`, `## Deviations`, `## Pending index`) plus `## What great looks like`: the
visual qualities and reference images you are aiming for. `## Craft principle` (the model source) and
`## How shipping games do it` must each **quote** the source they rest on, not paraphrase it from
memory.

For visible work the note also carries `## Reference set`: one entry per reference, with its URL, the
film (or source), its licence, and what it shows that the crate must reproduce: fresh against drying
blood colour, viscosity and how drops bead and run, the spray cone and droplet size spread, how a pool
spreads and edges, how a wound's edge, depth and sheen read, how fracture surfaces and debris look.
Those observations **are** the section 3 reference checklist and the GREAT / NOT YET bar in section 7.
**The note cites URLs only; the images stay under `/tmp/cN/refs/` and are never committed.** An image
may be committed only when its licence allows it (CC0 or CC-BY, from Openverse or Wikimedia Commons),
with its attribution beside it. The goal is to replicate the look procedurally, never to ship the
images.

Any changed default goes under `Deviations` with a citation, or is marked a design default. Commit the
note alone before writing code: `git add docs/research/<system>.md && git commit -m "Research: <system>"`.

## 6. Implement within the crate's rules

- Keep every rule in `CLAUDE.md`. In particular: `tests/leaf.rs` is the ratchet (it holds the library to
  its declared `[dependencies]`, `bloodstain` to no `bevy`, no `glam` and `[f32; 3]` signatures, and
  `src/` to no panicking call); widening it is a design decision recorded as one.
- **Bevy is 0.20.** The manifest moved in 0.6.0 (`AG-138`); the "Bevy 0.19 is pinned" lines in
  `CLAUDE.md` predate it. Read the vendored source (`~/.cargo/registry/src/index.crates.io-*/bevy-0.20.*/`),
  not bevy.org. A missing `Res<T>` panics its system, and every run condition is evaluated.
- **The bake is reproducible, or it does not happen.** Never seed from an `AssetId`, an entity, a clock
  or an accumulator: seeds are places quantised onto `WELD`. Sort the vertex soup before appending. No RNG
  dependency; `hash_f32` is the only random source. Ticks, not seconds (the physical-time models step by
  an integer count). `WoundKind` and `PatternClass` are append-only.
- **The goldens are locks, not snapshots.** A red golden with the build profile fixed means the model
  moved; that is the finding. A deliberate model change re-blesses its golden in the same commit, with
  the reason in its doc comment, and is named in the log and the changelog.
- **The CPU is the authority; the GPU is write-only.** Nothing reads `Assets<Image>`; digests fold the
  CPU buffers. The `flesh` material is cosmetic: nothing in a shader may be needed by a hash. A look
  matched to a reference is matched in the shader, the canvases' `shade`/`appearance` and the parameters,
  never by reading pixels back.
- **WebGL2 is a requirement.** Sixteen-byte-aligned uniforms, no storage buffers, at most sixteen
  textures with an environment map, forward path only. After touching `flesh.wesl` or
  `flesh_prepass.wesl`, re-check it in a browser as `CLAUDE.md` describes.
- No `unwrap()`, no `expect` on caller data, no panicking index. Malformed input is `warn!`-skipped or
  refused with `None`, `0` or `false`, never a plausible guess.
- A tuned constant says so, in its doc comment and in the citations note; a number from a paper names
  it. A colour picked to match a horror-film reference is a tuned constant: its doc comment names the
  reference set's note.
- The caller owns the schedule; `preset` is the one place that owns one (`GoreClock`, `FixedUpdate`).
- Plain functions with their own tests for any new maths, testable without an app.
- Re-export new public items from the crate root; `#![warn(missing_docs)]` holds.
- One path per feature, no fallbacks or shims; delete the code the change replaces.
- Tests: for a bug, a test that fails before and passes after. Permanent tests check what a caller sees
  (digests, conservation, refusals, topology); throwaway checks go in `/tmp`.
- Commit titles carry the next ticket id, continuing `BACKLOG.md`'s series (the last is `AG-139`).

## 7. Verify and sign off: it must be RIGHT, CONVINCING and FAST

You are grading your own work, so grade it as a hostile reviewer would: an effects artist holding your
frame beside the film still, and an engineer who will run the recorder twice and compare digests before
reading anything you are proud of.

- Rebuild, re-run the bench three times, the recorders twice each and the gore checklist under
  `/tmp/cN/after/`, and compare every line with the "before".
- **Compare against the references, side by side.** For every visible change build one sheet per
  phenomenon: `magick montage /tmp/cN/refs/<ref>.jpg /tmp/cN/before/<shot>.png /tmp/cN/after/<shot>.png -tile 3x1 -geometry 640x480+4+4 -label '%f' /tmp/cN/compare-<phenomenon>.png`,
  and read each sheet yourself, full size and zoomed on the blood.
- Write the critique BEFORE the verdict (`/tmp/cN/critique.md`): every flaw you can find, against the
  research note's `## Reference set`, `## What great looks like` and `## Validation targets`. **Each gap is
  named against a reference:** "against ref 3 (*The Thing*, URL) the pool has no dark drying ring", not
  "the pool could be better". Look for what is wrong, not for confirmation that your change worked.
- Then give each feature exactly one verdict: GREAT, NOT YET or FAIL. GREAT means a game could ship on it
  today and the frame holds up beside its references at a glance. "Fine", "acceptable" and "good" are NOT
  YET.
- On NOT YET or FAIL, write the specific changes that would move it to GREAT, apply them, rebuild,
  re-measure and re-judge. Do not lower the bar to pass.
- After 5 failed rounds on one feature, stop: log the blocker, the remaining flaws and your best
  hypothesis, and revert to the last GREAT state.
- **Reproducibility gate:** each recorder variant's two runs print the same digest; every digest the
  item did not mean to move equals the baseline's; every golden in section 3 passes untouched or was
  re-blessed deliberately in the same commit, with the reason.
- **Performance gate**, equal in weight: median of the three `carnage_frame_ms` and `carnage_p99_ms`
  values before and after, same machine; no regression over 5% without a logged, justified trade. A
  change that means to move the carnage (more droplets, a new stain) moves the bench's digest; the bench
  then refuses a timing, so the pass re-pins its digest in the same commit with the reason, and compares
  the timing against the baseline worktree's bench.
- Run the full gate (`.github/workflows/ci.yml` and `CLAUDE.md`, "Build and test"):
  1. `cargo build` (declared features only, no dev-dependencies: the only build that proves the crate
     compiles with exactly the features a consumer gets)
  2. `cargo build --release` (load-bearing, `CLAUDE.md`)
  3. `cargo nextest run` and `cargo test --doc` (nextest does not run doctests)
  4. `cargo build --examples`

  Clippy, rustfmt and wasm are not gates here (CI runs none of them, and clippy has a standing backlog of
  warnings); add no new clippy warning in a file you touch, and run `cargo fmt` on the files you touch.
  `--all-features` is not a gate until `vfx` builds on Bevy 0.20.

Fast testing across the home cloud — the host matrix, mold, sccache and the measured numbers — is `agents/TESTING.md` in the codex_fs depot: `/mnt/codex_fs/agents/TESTING.md`, or `ssh three cat /mnt/codex_fs/agents/TESTING.md` from a host that does not mount it.

## 8. Ship

Start only after section 7 passes. Then release (the "Release" block at the end of this section).

1. Document: rustdoc for every changed public item, the README for user-visible changes, the research
   note's `Deviations` and `Pending index`, and an entry under `## [Unreleased]` in `CHANGELOG.md`
   (`Added`, `Changed`, `Fixed`, `Removed`).
2. Regenerate the `docs/*.gif` whose behaviour has moved, from its recorder, with `tools/gif.sh` and the
   recipe in `docs/DEMOS.md`.
3. Log: append `## Iteration N — <date> — <title>` to `docs/loop-log.md`:
   - what changed and why it ranked first (and the runner-up)
   - the sources used: the physics, horror-film reference (URLs) and technique sources
   - your critique and verdict per capture, with evidence filenames
   - the gore checklist's numbers, before and after
   - the bench's medians and p99s, before and after, and the recorder digests
   - the commit hash, and any sibling-repo commits
   - the top 5 remaining items, ranked, for the next iteration
4. Branch and commit: `git checkout -b loop/iter-N` from `main`. Commit by path with messages that
   state the user-visible change: the research note (already committed in section 5), the
   implementation, the docs and changelog, then the log (`Loop log: iteration N, <title>`).
   `git status --short` must be empty afterwards.
5. Push and merge: `git push -u origin loop/iter-N`, then `git checkout main && git pull --ff-only &&
   git merge --no-ff loop/iter-N -m "Merge loop/iter-N: <title>"`. On a conflict, `git merge --abort`,
   log it, and stop.
6. Re-run the gate on the merged tree; a failure stops the pass before the push. Then
   `git push origin main`.

**Release, every pass that changed the package** (owner, 2026-10-29: loops publish, and never wait on CI to do it). After the push, from that pushed commit, on a clean tree with the gate green: if a file `cargo package --list` ships changed since the last published version (`git diff --quiet v<last> HEAD -- <those files>` fails; with no tag `v<last>`, diff from the commit that set that version in `Cargo.toml`, found with `git log -S'version = "<last>"' -- Cargo.toml`), release:
1. Decide the version with `cargo semver-checks check-release` (`cargo install cargo-semver-checks --locked` if missing). Pre-1.0: a breaking change bumps MINOR and resets PATCH, anything else bumps PATCH; never 1.0.0. Move `## [Unreleased]` in `CHANGELOG.md` to `## [<version>] - <date>`, bump `Cargo.toml` and `Cargo.lock`, and commit `release: v<version>`.
2. `cargo publish --dry-run`, then `cargo publish`, locally, with the credentials `cargo login` stored; never print them. Never wait for, watch or re-run a GitHub workflow to release.
3. `git tag -a v<version> -m v<version>`, then push the commit and the tag (merge, never force, on a refused push).
4. `cargo info <crate>@<version>` resolves; put the crates.io URL in the log entry.

**Every pass that changed the package releases it** (owner, 2026-10-31, replacing the six-hour limit of 2026-10-30). Release in the same pass as the change, after the push. A publish refused with `429` is not retried in this pass: log it once and go on; the next pass releases, and the audit on `loop-stats` shows it as `missing: release` until then. Any other failed publish is logged with its error and retried by the next pass.

### README, demos and docs stay true

A README is read before anything else, so a false line there costs more than a false line anywhere else.
An audit on 2026-10-27 found one in each of the four loop repos: a `tour.gif` captioned "at noon" over a HUD
reading 10.00 h with a running clock; a `ties.gif` captioned for rivalry colours that appear in none of its 240
frames; a hero caption promising an effect its file never shows; and a demo video captioned as the full
recording of the GIF above it when it shows different objects. Before you push a change a user can see:

1. **Put it in the section it belongs to.** Do not append to the introduction or to the end, and do not
   restructure the README in a feature pass. It reads in this order — what it is in one line, the demo, what
   you get, install, quick start, then the detail sections — and an item added in the wrong place undoes it.
2. **A caption describes the file as it exists.** Open the GIF or still (frame count, size, any HUD burned into
   it) and write only what you can read off it; a figure quoted from program output comes from running that
   program now, not from the log. Recipe: the example's capture knob (section 2).
3. **Media you cannot regenerate here is logged, not left lying.** With no GPU or display on this machine, name
   the stale file and its path in the report and change its caption to what it still shows.
4. **The README's code compiles.** Edit a README snippet only together with the code it shows, and check it against the crate-root doctest it mirrors.

## 9. Clean up when you are done

Worktrees and build directories outside the clone are not free: a `target/` for this crate is tens of GB, a
loop host's `/tmp` may be RAM, and on 2026-10-27 earlier passes had left 37 GB of baseline builds and 5 GB of scratch behind.
**After you ship and before you write the pass's last log line, whether or not you shipped,** remove what *you*
made this pass, and list what earlier passes left.

1. **A baseline is a pair that can be found.** Make it with `git worktree add --detach <path> <sha>` (never a
   branch name: a worktree with a branch checked out stops anything else checking that branch out) and build it
   into `<path>-target`. Then `git worktree list`, plus the `*-target` directory beside each worktree, is the
   whole inventory.
2. **Remove yours.** `git worktree remove <path>` (plain, never `--force`) and `rm -rf <path>-target`, then your
   scratch under `/tmp` (captures, frames, venvs, gate logs, `rustdoctest*`). Remove only what you created this
   pass, and name each path in the log. If git refuses because the worktree has a change in it, name the file
   in the log and leave it.
3. **Leftovers from earlier passes are listed, never deleted.** A killed pass cannot clean up after itself, but
   an agent does not delete what it did not create, so list them once and let a person approve the removal:

   ```sh
   git worktree list --porcelain | awk '/^worktree /{sub(/^worktree /,""); print}' | tail -n +2   # all but the clone
   du -sh <wt> <wt>-target                       # the size of each one that is not yours
   git -C <wt> status --short                    # empty: nothing is lost by removing it
   find <wt> <wt>-target -mmin -120 -print -quit # empty: untouched for two hours, so no live build is using it
   ```

   Log each with its path, size, and whether it is clean, already on `origin/main` and idle (the likeliest to be
   stale). Say so, and still leave it.
4. **Never remove** the clone itself, its own `target/` (the build cache the next pass starts from), anything you
   did not create this pass, or anything a process is using.
5. Write one log line: what you removed and the space it freed (`df -h .` before and after), and the leftovers
   you listed.

## Rules

- One improvement per iteration, fully finished: researched, implemented, GREAT-signed-off,
  benchmarked, gated, committed, documented, logged.
- No feature ships without a written critique, a GREAT verdict and a clean performance check.
- If blocked, log the blocker and pick the next item. A failed gate or a visible defect is not a
  blocker: fix it. A failed push or merge is: log the exact error and stop. Never force-push.

README and docs follow https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md; a feature is not done until its README line and, if visible, its GIF exist.
