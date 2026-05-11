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
- `world/`: camera setup/control, fixed demo layout, tavern map markers, tile spawning, wall/furniture/station placement.
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
- The game simulates on a 2D grid or 2D world plane.
- The view is fixed isometric.
- Depth is created through sprite anchors, Y-sorting, layered assets, shadows, and occlusion.
- Buildings are assembled from floor, wall, prop, roof, and overlay sprites.

It does not mean true 3D models with a perspective camera.

## Coordinate Systems

Use separate concepts:

- `GridPos`: logical tile position, usually integer or half-tile coordinates.
- `WorldPos`: continuous 2D simulation position.
- `IsoPos`: screen-space render position derived from world/grid coordinates.
- `Depth`: render ordering value derived from position and layer.

The conversion should stay centralized. Gameplay systems should not manually calculate screen positions.

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

## Collision And Walkability

Collision should be grid-driven first:

- Each tile has walkable or blocked state.
- Props can add blocked cells.
- Doors can toggle blocked/open state.
- Some props can be interactable without blocking movement.

Later, add local avoidance for NPCs if needed. Do not start with complex physics.

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

Floors should be 1-based in design data:

- Floor 1, Floor 2, and so on.
- Higher floors are visually offset, not sorted by one-off overrides.
- Only the current floor should have active wall collision.
- Other floors can be faded, hidden, or shown according to inside/outside state.

This is long-term work, but the foundation should not block it.

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
- `stations.ron`
- `customers.ron`
- `events.ron`
- `rooms.ron`
- `maps/*.ron`
- `workers.ron`
- `factions.ron`
- `jobs.ron`
- `rumors.ron`
- `upgrades.ron`

RON is a natural fit for Rust, but JSON is acceptable if we want easier editing in external tools.

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

- `WorldMap` resource for tile/object data.
- Spawn systems that read map data.
- Asset registry for sprite handles and metadata.
- Shared function for render transform and depth.
- Runtime debug overlays.
- Data-driven stations and NPC archetypes.

Keep gameplay playable while replacing foundation pieces. Avoid a long rewrite that leaves no runnable build.
