# Visual Upgrade Plan

This plan exists because the current game is mechanically playable but still reads as a prototype: simple tiles, transparent wall blocks, static characters, and colored placeholder props. The next visual work should make the scene feel like a small game while preserving the clean data-driven structure already built.

Use `data/art_direction.md` as the art director. Existing concept references are source material for style, proportion, palette, and object density:

- `data/characters.png`: 8-direction character concepts.
- `data/environments.png`: exterior, interior, wall, roof, ground, and prop material reference.
- `data/concepts/last_call_interior.png`: target tavern mood and density.
- `data/concepts/survivor_sprite_concepts.png`: character style and outfit language.

If an asset is missing, create or generate a new one that matches those documents. Do not let missing source art block the pipeline.

## Visual Milestone A: Asset Inventory And Runtime Metadata

Goal: Know exactly which prototype visuals must be replaced and define how runtime assets are addressed.

Deliverables:

- Inventory current runtime assets in `assets/characters/` and `assets/tiles/`.
- List missing runtime folders: `assets/walls/`, `assets/roofs/`, `assets/props/`, `assets/decals/`, and animation sheets.
- Define stable visual asset ids for tiles, walls, roof pieces, props, and character clips.
- Decide source-to-runtime workflow for concept-sheet extraction and generated replacements.
- Keep anchors, layers, footprints, and cutaway behavior outside the image files.

Done when:

- Every current colored mesh or placeholder sprite has a target replacement category.
- RON data can refer to stable visual ids instead of raw color blocks for world art.

## Visual Milestone B: Character Animation Pipeline

Goal: Support 8-direction animated characters without putting animation rules in player or NPC movement code.

Deliverables:

- Components for `FacingDirection`, `AnimationState`, and current animation clip.
- Direction resolver from movement vector to 8 directions.
- Frame timer and frame selection system.
- Character visual metadata for idle and walk clips.
- Main character idle/walk in 8 directions.
- At least three NPC archetypes idle/walk in 8 directions.

Done when:

- The player animates while moving north, northeast, east, southeast, south, southwest, west, and northwest.
- NPCs animate from the same shared animation systems.
- Adding a new character means adding metadata and art, not new movement code.

## Visual Milestone C: Building Kit Replacement

Goal: Replace transparent wall rectangles and flat roof placeholders with readable 2.5D building pieces.

Deliverables:

- Opaque wall sprites for back wall and side wall runs.
- Corner, post, door, window, trim, and damaged-wall pieces.
- Roof pieces built from patched wood and corrugated metal style references.
- Cutaway-compatible roof and upper wall behavior.
- Anchor presets strong enough to avoid `CustomYOffset` for normal wall and roof assets.

Done when:

- The tavern interior reads as a real building with walls, corners, roof logic, and material detail.
- Interior visibility still works when the player enters the room.
- Prototype wall transparency is gone.

## Visual Milestone D: Tile, Prop, And Decoration Pass

Goal: Replace flat prototype objects with dense, readable tavern art.

Deliverables:

- Worn plank floor variants.
- Wasteland dirt and porch variants.
- Bar counter, table, stool, stove, pantry, water still, radio, barricade, crate, shelf, lamp, rug, and bedroll sprites.
- Decals for grime, floor wear, papers, bottle clutter, soot, spill marks, cracks, and warm light.
- Generated shadows or authored shadow sprites for large props.

Done when:

- The screenshot no longer reads as a floor grid with colored blocks.
- Stations are visually identifiable before reading the prompt text.
- The room has enough clutter and material variation to match the concept references.

## Visual Milestone E: Interaction Feedback And Game Feel

Goal: Make the upgraded visuals respond to play.

Deliverables:

- Stove warmth or flame feedback.
- Radio pulse or antenna feedback.
- Water still activity feedback.
- Service pose or short interaction feedback when serving customers.
- Spill cleanup feedback.
- Angry/hungry customer visual state placeholder.
- UI readability pass over denser art.

Done when:

- Important gameplay state is visible in the scene, not only in the HUD.
- A 5-minute session feels animated and readable.

## Bottlenecks

- The current character runtime uses one still PNG per character, so animation metadata and frame selection must be built before 8-direction sheets are useful.
- Existing wall, roof, and prop visuals are mesh rectangles, so the world renderer needs sprite-backed prop visuals before real building art can replace them cleanly.
- Concept sheets are not automatically sliced runtime assets. They need extraction, cleanup, transparent backgrounds, and consistent pivots.
- Current world data still contains hand-tuned `CustomYOffset` values for walls and roof. Production assets need reliable anchor metadata.
- Visual verification still depends on user screenshots when this assistant session cannot open a Bevy window.
