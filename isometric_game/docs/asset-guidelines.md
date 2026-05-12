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
- `assets/decals/`
- `assets/items/`
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

Directional suffixes must follow the canonical isometric direction rule in [Technical Foundation](technical-foundation.md), not keyboard labels:

- `_n`: up-left isometric axis.
- `_e`: up-right isometric axis.
- `_s`: down-right isometric axis.
- `_w`: down-left isometric axis.
- `_ne`: visual up.
- `_se`: visual right.
- `_sw`: visual down.
- `_nw`: visual left.

For prototype assets that already exist, short names such as `tavern_floor.png` are acceptable. New production-facing assets should follow the longer form.

## Prompt Provenance

Generated concept images that become art-direction references should keep their exact prompt beside the image as a sidecar file:

```text
data/concepts/<image_name>.png
data/concepts/<image_name>.prompt.md
```

The sidecar should preserve the prompt that produced the accepted image, not a later summary. This lets future image generation reuse successful wording without depending on chat history.

## Anchor Rules

Every visual asset must have an anchor rule. The anchor describes how the visual is attached to its logical grid position.

Current anchor presets:

- `TileCenter`: floor and ground tiles. The grid coordinate is the center of the diamond.
- `LowProp`: tables, stools, barricades, crates, and station bodies. The visual is lifted slightly above the grid point.
- `WallBaseline`: wall pieces. The grid coordinate is the wall baseline.
- `RoofCover`: roof pieces and upper building covers. The grid coordinate is the building footprint anchor.
- `Foot`: characters. The grid coordinate is the character foot position.
- `CustomYOffset(value)`: temporary escape hatch for unusual one-off prototype objects.

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

## Runtime Sprite Sources

Concept sheets can be converted into runtime sprite candidates with:

```bash
cargo run --bin extract_concept_sprites
cargo run --bin extract_character_directions
cargo run --bin import_guided_turnaround
cargo run --bin import_walk_cycles
```

The concept extractor writes transparent PNG candidates into `assets/props/`, `assets/walls/`, `assets/roofs/`, `assets/tiles/`, `assets/items/`, and `assets/decals/`. It removes only neutral background connected to the crop border, then applies a short-gap silhouette close so outlines, wood fills, and internal shadows stay fully opaque instead of becoming see-through.

The character direction extractor slices the first character row from `data/characters.png` into `assets/characters/traveler_a_raw/` as source-column candidates. Do not assign canonical direction suffixes from source labels or prompt column order. Generated source sheets can have incorrect labels, swapped rotations, or repeated poses, so a human-visible direction audit must happen before files are renamed to `_n`, `_ne`, `_e`, `_se`, `_s`, `_sw`, `_w`, or `_nw`.

The guided-turnaround importer consumes human-approved rotation sheets configured in `data/actors/turnaround_imports.ron`. Each import records the source column, degree label, and canonical direction suffix, then writes normalized transparent runtime frames such as `assets/characters/bartender_turnaround/idle_se.png`. The importer validates that all eight direction suffixes are present exactly once and that every output has both transparent and opaque pixels.

The walk-cycle importer consumes six-frame one-direction strips configured in `data/actors/walk_cycle_imports.ron`. It removes the guide area, converts chroma green to alpha, trims the visible character, and bottom-aligns each frame onto a stable transparent canvas before writing files such as `assets/characters/bartender_walk/walk_se_03.png`.

## Cutaway Rules

Building cover sprites can declare cutaway behavior in world data.

Current rule:

```ron
cutaway: HideWhenPlayerInside(zone: "main_tavern")
```

The referenced zone must be defined in the same world blueprint. Keep cutaway metadata on the placed object or reusable style, not in renderer-specific code.
