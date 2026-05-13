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

## Dependency Stack

Build the authoring system in this order:

1. Asset registry with stable IDs.
   - `data/assets/registry.ron` maps stable sprite IDs to current file paths and editor palette kinds.
   - World and prefab files should refer to `asset_id`, not raw PNG paths.
   - Raw `asset` paths remain only as a legacy fallback for old prototype files.
2. Prefab/archetype system.
   - Prefabs define reusable object defaults: visual asset id, layer, anchor, size, depth offset, collision footprint, cutaway rule, and optional station behavior.
   - World files place prefab instances and override only the fields that differ.
3. In-game scene editor.
   - The editor selects registry IDs and prefab IDs, creates placed instances, edits overrides, validates the scene, and saves `.ron`.
   - Placed objects need stable instance IDs before reliable move/delete/duplicate, undo/redo, and external editing.
4. Hot reload world/assets.
   - Reload registry, prefab, and world data into a staging copy first.
   - Validate IDs, footprints, stations, markers, and overlaps before replacing the live scene.

This order matters: prefabs, editor actions, and hot reload all become fragile if objects are still identified by mutable file paths or list positions.

## Editor V0 Scope

V0 should be an in-game authoring mode, not a separate professional tool.

Required workflow:

```text
cargo run --bin isometric_game -- --editor
cargo run --bin isometric_game -- --world data/worlds/demo_tavern.ron --editor
```

Running editor mode without `--world` starts from an untitled empty scene and opens the scene selector first. Existing scenes must be loaded deliberately from the selector or by passing `--world`; the current tavern scene is not the default first look for authoring.

The user can:

- start a zero-base scene with no ground tiles
- load an existing scene from the scene selector
- select a prefab id from a simple palette
- place it on the grid
- move, duplicate, and delete it
- edit basic metadata
- save the world back to `.ron`
- reload the world without recompiling

Current controls:

- `F10`: toggle editor mode.
- The scene selector appears first in `--editor` mode without `--world`; the toolbar `Scenes` button reopens it.
- Toolbar buttons select object, tile, edge-wall, marker, vertical-link, zone, collision-cell, and world tools.
- Toolbar action buttons apply, select, move, copy, delete, clear to an empty scene, cycle brushes, rotate/type, toggle two-way links, validate, save, and reload.
- Left-click in the scene applies the active tool at the clicked cell.
- Right-click in the scene selects the nearest editable element for the active tool.
- `PageUp` / `PageDown`: change the active `z` level.
- `F8`: validate the current world document.
- `F9`: validate, then save the current world document.
- `F5`: hot reload the asset registry, prefab catalog, and current world file from disk.

Bevy's `file_watcher` feature is enabled for runtime art files. When the platform supports it, changed PNGs under `assets/` can refresh through the asset server; `F5` is still the authoritative path for validating and applying registry, prefab, and world document changes.

The editor now edits prefab instances, tile overrides, explicit collision cells, edge walls, z-aware seat/entrance markers, vertical links, zones, and world level count. The `Empty` button clears scene content while preserving the style library, so a scene can start with no ground tiles. Legacy `prop_styles`, `props`, `prop_runs`, and `stations` still load for compatibility, but they are not yet fully inspector-editable.

## Prefab Data

Reusable world objects live in:

```text
data/prefabs/world_objects.ron
```

A prefab owns the reusable defaults for an object:

```ron
(
    id: "prop.table_round",
    label: "Round table",
    style: (
        size: [72.0, 58.0],
        asset_id: "prop.table_round",
        layer: Prop,
        anchor: LowProp,
        depth_offset: 36.0,
        shadow: true,
        footprint: Nearest,
    ),
)
```

Station-ready prefabs declare station behavior:

```ron
station: Some(Stove),
```

World files place prefab instances with stable instance IDs and optional `z`:

```ron
prefab_instances: [
    (id: "table_001", prefab: "prop.table_round", grid: [10.0, 9.0], z: 0),
    (id: "upper_table_001", prefab: "prop.table_round", grid: [10.0, 9.0], z: 1),
]
```

Changing a prefab updates future reloads of every instance that references it. Changing an asset path should happen in `data/assets/registry.ron`, not in world files.

World documents can declare level count:

```ron
levels: 2,
```

`z = 0` is the ground level in engine data. The editor currently displays the raw engine `z` value; later UI can label it as Floor 1, Floor 2, basement, roof, and so on.

Navigation markers are z-aware. Existing two-number marker syntax still means `z: 0`; detailed marker syntax is available for upper floors:

```ron
seats: [
    [11.2, 8.9],
    (grid: [11.2, 8.9], z: 1),
],
entrance: (grid: [20.8, 9.0], z: 0),
```

Vertical links connect navigation levels explicitly:

```ron
vertical_links: [
    (
        id: "stairs.main",
        kind: Stairs,
        lower: (grid: [10.0, 10.0], z: 0),
        upper: (grid: [10.0, 10.0], z: 1),
        bidirectional: true,
    ),
]
```

Edge walls block movement across a cell boundary without blocking the cells themselves:

```ron
edge_walls: [
    (grid: [5, 4], z: 0, direction: S, style: "back_wall"),
]
```

Explicit collision cells can block or clear walkability at a grid cell without adding a visible prop:

```ron
collision_cells: [
    (grid: [9, 7], z: 0, blocked: true),
]
```

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
