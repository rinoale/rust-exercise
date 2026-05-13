# Technical Foundation

This document describes the intended engine layer for the 2.5D isometric tavern game in Rust and Bevy.

## Engine Choice

The project uses Bevy because:

- Rust is part of the learning and engineering goal.
- ECS fits simulation-heavy tavern management.
- Custom rendering, world data, and save systems are acceptable project work.
- Time is not the main constraint.

The tradeoff is that Bevy does not provide a finished isometric survival-game pipeline. We will build the missing layer ourselves.

## Current Source Layout

`src/main.rs` is only the Bevy app composition root. It wires plugins and system sets, but it should not own gameplay rules, map construction, actors, rendering helpers, or UI logic.

Current module responsibilities:

- `config.rs`: prototype constants shared across modules.
- `universe/`: coordinate conversion, render depth, logical grid position, render layers, map walkability, and debug overlays.
- `world/`: camera setup/control, RON world import, reusable world blueprints, fallback demo layout construction, tavern map markers, tile spawning, wall/furniture/station placement.
- `actors/`: moving entities such as player and customers, including movement, spawning, patience, and actor-specific state.
- `gameplay/`: non-visual rules: tavern state, orders, stations, interactions, spills, resource changes, notices, and future event logic.
- `rendering/`: reusable sprite and mesh construction helpers.
- `ui/`: HUD and contextual prompt rendering. UI reads gameplay state; it should not advance simulation.

Dependency direction should stay mostly one way:

```text
main -> plugins
actors/world/gameplay/ui -> universe/rendering/config
ui -> actors/gameplay/world state queries
gameplay -> actors/world facts when resolving rules
```

If a feature becomes shared by many systems, place the stable primitive in `universe` only when it is about space, identity, time, layers, or low-level simulation rules. Place tavern-specific business rules in `gameplay`.

## 2.5D Definition For This Project

2.5D means:

- The world uses 2D sprites and tile assets.
- The game simulates on an isometric grid with discrete `z` levels.
- The view is fixed isometric.
- Depth is created through sprite anchors, Y-sorting, layered assets, shadows, and occlusion.
- Buildings are assembled from floor, wall, prop, roof, and overlay sprites.

It does not mean true 3D models with a perspective camera.

## Coordinate Systems

Use separate concepts:

- `GridPosition`: logical position with continuous `x/y` and discrete `z` level.
- `WorldPos`: continuous 2D simulation position.
- `IsoPos`: screen-space render position derived from world/grid coordinates.
- `Depth`: render ordering value derived from position and layer.

The conversion should stay centralized. Gameplay systems should not manually calculate screen positions.

Important rule:

```text
world z/elevation != Bevy Transform.z
```

World `z` is the logical building level used by collision, placement, cutaway, and future pathfinding. Bevy `Transform.z` remains a renderer depth value derived from `(x, y, z, layer, depth_offset)`.

Current projection:

```text
screen_x = (grid_x - grid_y) * tile_width / 2
screen_y = -(grid_x + grid_y) * tile_height / 2 + z * level_height
depth    = z * level_depth_stride + y_sort(x, y) + layer_depth + local_depth_offset
```

Current level indexing is zero-based in engine data: `z = 0` is ground floor, `z = 1` is the next level. User-facing editor labels may later display these as Floor 1, Floor 2, and so on.

## Canonical Isometric Directions

Direction names are based on the isometric world/object axes, not keyboard labels and not raw screen up/down/left/right.

The project uses this canonical compass:

```text
N     E
 \   /
  \ /
  / \
 /   \
W     S
```

With the current grid projection:

```text
screen_x = (grid_x - grid_y) * tile_width / 2
screen_y = -(grid_x + grid_y) * tile_height / 2
```

the direction mapping is:

| Iso direction | Visual screen direction | Grid delta |
| --- | --- | --- |
| `North` | up-left | `(-1, 0)` |
| `East` | up-right | `(0, -1)` |
| `South` | down-right | `(1, 0)` |
| `West` | down-left | `(0, 1)` |
| `NorthEast` | up | `(-1, -1)` |
| `SouthEast` | right | `(1, -1)` |
| `SouthWest` | down | `(1, 1)` |
| `NorthWest` | left | `(-1, 1)` |

This rule is owned by `universe::IsoDirection`. Character facing, wall/object sprite suffixes, pathfinding directions, and future placement tools should use this same direction type.

For current controls, `W` moves visually up, so it resolves to `NorthEast`; `D` resolves to `SouthEast`; `S` resolves to `SouthWest`; `A` resolves to `NorthWest`.

Source art labels are not authoritative. Generated sheets can contain incorrect direction labels or repeated poses, so keep raw candidate names until the actual pose direction is audited from the pixels.

For new generated character sheets, prefer angle-based prompts over compass words:

| Rotation column | Visual facing | Project direction |
| --- | --- | --- |
| `0 deg` | screen up / away | `NorthEast` |
| `45 deg` | up-right | `East` |
| `90 deg` | screen right | `SouthEast` |
| `135 deg` | down-right | `South` |
| `180 deg` | screen down / toward | `SouthWest` |
| `225 deg` | down-left | `West` |
| `270 deg` | screen left | `NorthWest` |
| `315 deg` | up-left | `North` |

Only rename raw candidates to `_n`, `_ne`, `_e`, `_se`, `_s`, `_sw`, `_w`, or `_nw` after a visual audit confirms every pose is distinct and correctly oriented. If the sheet lacks all eight directions, do not wire it as an 8-direction runtime character.

Prompt compliance is not a direction guarantee. For production-quality animation, prefer one of these workflows:

- Deterministic: model or rig the character once, then render the eight rotations from the same camera.
- Assisted: generate rotation candidates, audit the actual visible facing direction, record an explicit source-column-to-`IsoDirection` mapping, then assemble the canonical runtime sheet from that mapping.
- Reject: if a generated candidate has missing, duplicated, or mirrored directions, keep it as concept art only.

## Render Layers

Initial layer order:

1. Ground terrain.
2. Floor tiles.
3. Low floor decals: stains, dirt, rugs, shadows.
4. Low props: dropped items, small debris.
5. Dynamic Y-sorted props and characters.
6. Tall walls and tall props.
7. Roof and cutaway overlays.
8. UI and debug overlays.

Dynamic props and characters should sort by their anchor position, not by image center.

## Universal Sorting Rule

The Unity prototype note defined a universal sorting rule:

```text
sortingOrder = -worldY * sortScale
```

In Bevy, the equivalent should be centralized as a depth function. Every object that can overlap must use the same depth rule, with small layer offsets only for controlled categories such as floor decals, walls, roofs, and UI.

Do not identify special cases by entity name. Sorting behavior should come from components and asset metadata.

## Sprite Anchors

Every asset type needs a defined anchor:

- Diamond floor tile: center of diamond.
- Character: feet position.
- Chair/table/barrel: floor contact point.
- Wall: bottom edge or wall-grid baseline.
- Door/window: wall baseline.
- Roof: building footprint anchor.

If anchors are inconsistent, 2.5D sorting will fail.

The active anchor presets are documented in [Asset Guidelines](asset-guidelines.md). World data should use the `anchor` field in `prop_styles` instead of encoding raw offsets in Rust code.

## Collision And Walkability

Collision should be grid-driven first:

- Each tile has walkable or blocked state.
- Props can add blocked cells.
- Doors can toggle blocked/open state.
- Some props can be interactable without blocking movement.

Later, add local avoidance for NPCs if needed. Do not start with complex physics.

Current implementation:

- `WorldMap` owns tile and prop blockage.
- `WorldMap` stores blockage per `(x, y, z)` level.
- `WorldMap` stores edge-wall blockage per `(x, y, z, edge)` so a wall can block crossing between two cells without occupying either cell.
- `universe::physics` owns shared kinematic movement, actor body radius checks, map clamping, and simple slide-on-collision response.
- Actor movement checks collision on the actor's current `z` level.
- Actor movement checks edge-wall crossings between adjacent rounded cells.
- Actors choose intent and speed, but they do not decide how collision is resolved.
- Gameplay rules should consume positions and outcomes, not reimplement movement.

Near-term physics additions should stay small:

- Door open/closed changes update `WorldMap`.
- NPC pathfinding asks the map for routes, then still moves through `universe::physics`.
- Dynamic local avoidance can layer on top of path intent later, but static world collision remains the base truth.

## Building Model

Represent a building as data:

- Floor layer.
- Wall layer.
- Stairs layer.
- Door/window layer.
- Prop layer.
- Roof layer.
- Room/zone metadata.
- Interaction markers: seats, queue points, stations, exits.

Early hardcoded placement is acceptable for experiments, but milestone work should move toward data-driven layouts.

## Roof And Cutaway Rules

The player must see the playable interior.

Initial rule:

- Hide or fade roof when the player enters the building footprint.
- Hide or fade front-facing walls when they block the player or important interactables.

Current implementation:

- World blueprint files can define named zones.
- Prop styles can declare `cutaway: HideWhenPlayerInside(zone: "...")`.
- The first demo tavern roof uses this rule and is hidden when the player is inside `main_tavern`.

Later rule:

- Per-room visibility based on player room.
- Show exterior walls for rooms the player is not inside.
- Keep low walls or wall silhouettes for orientation.

## Inside And Outside Detection

The Unity note used flood fill from wall tilemap edges:

- Cells reachable from outside edges are outside.
- Enclosed cells are inside.
- Inside/outside should update when walls change.

For Bevy, keep the same concept but implement it in map data, not renderer-specific code. The visibility system should consume room/zone data and player position.

## Multi-Floor Direction

The universe now supports discrete `z` levels:

- `GridPosition` carries `z`.
- `WorldMap` stores collision per level.
- World data can declare explicit collision cells for editor-authored blockers or walkability overrides.
- `WorldMap` stores edge walls per level and direction.
- World documents can declare `levels`.
- Tile regions, props, stations, zones, and prefab instances can declare `z`.
- Seat and entrance markers are `NavPoint { value, z }`, not raw `Vec2`.
- World documents can declare vertical links such as stairs and ladders.
- `E` traverses a nearby vertical link between its endpoints.
- Customers can route to a destination on another level through a direct vertical link.
- World visibility hides levels above the active player/editor level, then applies cutaway rules.
- Higher floors are visually offset, not sorted by one-off overrides.

Remaining multi-floor work:

- Multi-step pathfinding across several vertical links.
- Door/window semantics on edge walls.
- Dedicated stair/ladder/railing visuals and editor palette categories.
- Better editor filtering controls for showing only current floor, current-and-below, or all floors.
- Basement and roof-specific visibility policies.

## Object Identity

Never identify runtime objects by file name or scene name. Use components and metadata:

- Walls: `WallTile` or wall layer metadata.
- Stairs: `FloorObject { kind: Stair }`.
- Stations: `StationKind`.
- Props: `PropKind`, footprint, and interaction metadata.
- Characters: `Npc`, `Worker`, `Customer`, `Player`.

Names are acceptable for asset paths and debugging labels, not gameplay logic.

## NPC Architecture

Recommended components:

- `Npc`
- `GridAgent`
- `Path`
- `ActorAnimation`
- `Customer`
- `Need`
- `Patience`
- `TraitSet`
- `CurrentTask`
- `FactionStanding`
- `RumorSource`

Recommended customer states:

- `Entering`
- `FindingSeat`
- `Waiting`
- `Consuming`
- `Paying`
- `Leaving`
- `Angry`

Worker states should be separate from customer states. Workers need jobs such as cooking, guarding, repairing, farming, scouting, hauling, and healing.

NPC behavior should be data-driven enough to add archetypes without changing pathfinding or station code.

## Actor Animation Pipeline

Character animation metadata lives in `data/actors/character_animations.ron`.

Current rule:

- Movement systems update `GridPosition`.
- `ActorVisualMotion` derives moving/idle state and 8-direction facing from position deltas.
- `ActorAnimation` chooses an animation clip from metadata and advances sprite frames.
- Existing character art is still placeholder single-frame art, but the runtime path already supports state/direction/frame selection.

Do not put animation selection in player or customer movement. Movement expresses intent and position; actor animation consumes the resulting state.

## Interaction Architecture

Recommended components:

- `Interactable`
- `InteractionRadius`
- `Station`
- `Inventory`
- `TaskDuration`
- `Prompt`

Interactions should be resolved by:

1. Find nearest valid interactable.
2. Check requirements.
3. Start or complete task.
4. Apply resource and state changes.
5. Emit UI notice/event.

## Data Files

Likely data files:

- `items.ron` or `items.json`
- `data/gameplay/station_actions.ron` for station costs, outputs, notices, and cooldowns
- `stations.ron` for future station archetypes if station visuals and actions split further
- `customers.ron`
- `data/actors/character_animations.ron`
- `events.ron`
- `rooms.ron`
- `data/worlds/*.ron` for world blueprints during the prototype phase
- `maps/*.ron` or `worlds/*.ron` after the final content folder convention is chosen
- `workers.ron`
- `factions.ron`
- `jobs.ron`
- `rumors.ron`
- `upgrades.ron`

RON is a natural fit for Rust, but JSON is acceptable if we want easier editing in external tools.

World blueprint files should prefer named `prop_styles` for repeated visual/collision settings. Individual placements should reference styles by name unless they are true one-off objects.

## Save Data

Save at least:

- Game time.
- Tavern resources.
- Tavern reputation/morale/threat.
- Map/building state.
- Active NPCs and queued events.
- Inventory/storage.
- Open/closed doors.
- Dirt/damage state.
- Worker roster and assignments.
- Faction standing and known rumors.
- Upgrade and construction state.

Save data should not depend on render entities. Store logical game state, then rebuild render entities on load.

## Debug Tools

Required early:

- Toggle grid overlay.
- Toggle collision overlay.
- Toggle interaction radius.
- Toggle depth/order labels.
- Show NPC state above characters.
- Print selected tile/object data.

These are not polish. They are necessary for building the 2.5D foundation without guessing.

## Near-Term Code Direction

The current prototype should gradually move from hardcoded rendering toward:

- `WorldBlueprint` as the source for tile/object placement data.
- `WorldMap` resource derived from blueprint walkability.
- Spawn systems that read map data.
- Asset registry for sprite handles and metadata.
- Shared function for render transform and depth.
- Runtime debug overlays.
- Data-driven stations and NPC archetypes.

Keep gameplay playable while replacing foundation pieces. Avoid a long rewrite that leaves no runnable build.
