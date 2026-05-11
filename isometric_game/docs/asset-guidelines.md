# Asset Guidelines

These rules keep 2.5D assets predictable in Bevy. The goal is that a sprite or mesh can be placed from data without per-object guessing in Rust code.

## Naming Convention

Use lowercase snake case for asset files.

Recommended folders:

- `assets/tiles/`
- `assets/characters/`
- `assets/props/`
- `assets/walls/`
- `assets/roofs/`
- `assets/ui/`

Recommended file shape:

```text
<domain>_<object>_<variant>_<view_or_state>.png
```

Examples:

```text
tile_tavern_floor_worn_a.png
tile_porch_wood_worn_a.png
wall_tavern_back_plank_a.png
prop_table_round_wood_a.png
prop_stove_rusty_a.png
character_bartender_idle_se.png
ui_icon_scrap_a.png
```

For prototype assets that already exist, short names such as `tavern_floor.png` are acceptable. New production-facing assets should follow the longer form.

## Anchor Rules

Every visual asset must have an anchor rule. The anchor describes how the visual is attached to its logical grid position.

Current anchor presets:

- `TileCenter`: floor and ground tiles. The grid coordinate is the center of the diamond.
- `LowProp`: tables, stools, barricades, crates, and station bodies. The visual is lifted slightly above the grid point.
- `WallBaseline`: wall pieces. The grid coordinate is the wall baseline.
- `CustomYOffset(value)`: temporary escape hatch for hand-tuned prototype objects, including early roof/cutaway placeholders.
- `Foot`: characters. The grid coordinate is the character foot position.

World data should prefer named anchors over raw offsets. Use `CustomYOffset` only when a prototype object visibly needs tuning.

## Size Rules

Record logical render size in data before tuning depth or collision.

- Floor tile: `64 x 32` for the current prototype.
- Character sprite: `48 x 100` for the current prototype.
- Props: size should describe the visible body, not the shadow.
- Shadows are generated separately by the world spawn path for mesh props.

## Collision Rules

Collision belongs to the world blueprint, not the image file.

- `footprint: None`: decoration only.
- `footprint: Nearest`: block the nearest grid cell.
- `footprint: Cells([...])`: explicit blocked cells for larger objects.

Do not infer collision from sprite dimensions. Large art can be decorative; small art can still block movement.

## Depth Rules

Depth is derived from grid position and `UniverseLayer`. Use `depth_offset` only for local ordering inside the same cell or style group.

Common layers:

- `Ground` and `Floor`: terrain.
- `Decal`: stains, shadows, rugs.
- `Prop`: furniture, counters, station bodies.
- `Wall`: wall pieces and tall blockers.
- `Character`: player and NPCs.
- `Roof`: roof pieces and upper building covers that can be hidden or faded.
- `Overlay`: debug or world-space overlays that must stay above normal scene objects.

Do not encode depth behavior in file names.

## Cutaway Rules

Building cover sprites can declare cutaway behavior in world data.

Current rule:

```ron
cutaway: HideWhenPlayerInside(zone: "main_tavern")
```

The referenced zone must be defined in the same world blueprint. Keep cutaway metadata on the placed object or reusable style, not in renderer-specific code.
