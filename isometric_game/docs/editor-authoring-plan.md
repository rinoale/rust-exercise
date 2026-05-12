# Editor Authoring Plan

The scene editor exists to keep content work out of Rust source code. The game should be able to load, edit, save, and reload scenes from data files while the engine code remains unchanged.

## Ownership Rule

Rust code owns:

- grid projection, render sorting, collision, animation playback, and interaction rules
- validation logic for world data
- editor tools and UI behavior

Data files own:

- scene layout
- placed tiles, walls, roofs, props, decals, stations, and markers
- sprite asset ids, anchors, render sizes, layer/depth offsets, collision footprints, and cutaway zones
- station ids and marker ids referenced by gameplay systems

If creating a new place requires editing Rust source, the editor milestone is not complete.

## Editor V0 Scope

V0 should be an in-game authoring mode, not a separate professional tool.

Required workflow:

```text
cargo run --bin isometric_game -- --world data/worlds/demo_tavern.ron --editor
```

The user can:

- select an asset id from a simple palette
- place it on the grid
- move, duplicate, and delete it
- edit basic metadata
- save the world back to `.ron`
- reload the world without recompiling

## First Tool Surfaces

- Grid cursor with current tile coordinate.
- Palette panel grouped by category: tile, wall, roof, prop, decal, station, marker.
- Inspector panel for selected object metadata.
- Debug overlays for collision, anchors, depth order, cutaway zones, and station radius.
- Save/reload commands.
- Validation output panel.

## Validation Rules

Before saving, the editor should report:

- missing or unknown asset id
- invalid layer or anchor
- invalid render size
- blocking footprint outside map bounds
- overlapping blocking objects
- station id that has no gameplay definition
- duplicated marker ids
- cutaway zone references that do not exist

Validation should be strict enough to avoid broken scenes, but it should not block experimental visual placement when gameplay fields are intentionally empty.

## Non-Goals For V0

- No full asset painting workflow.
- No animation editor.
- No terrain sculpting.
- No external GUI framework unless Bevy UI becomes a blocker.
- No final UX polish. The first editor is a development tool.

## Why This Comes Next

The project now has enough engine skeleton to make content manually: data-driven world import, sprite-backed props, collision, cutaway, animation metadata, and runtime validation tests. The bottleneck is authoring speed. Without an editor, every new scene becomes a source-code or hand-written-RON task. That will slow world building and make visual iteration too expensive.
