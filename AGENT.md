# Agent Rules

This repository can contain several Rust exercises, but the main active game project is `isometric_game`.

## File Placement Rule

Before creating any new source file for `isometric_game`, classify it by the six basic domains below. Put the file in the matching module. Do not add new gameplay, world, actor, UI, universe, or rendering logic to `src/main.rs`; `main.rs` is only the Bevy app composition root.

## Six Basic Domains

1. `universe/`
   - Foundational rules that every system can rely on.
   - Coordinate conversion, grid/world position, render depth, layers, walkability primitives, debug overlays, identity/time primitives if they become shared.
   - Must stay generic. Do not put tavern-specific economy, customer, station, story, or room behavior here.

2. `world/`
   - Physical placed space.
   - Maps, terrain, floors, walls, rooms, furniture placement, building layout, camera tied to world navigation, spawn markers, entrances, seats, zones.
   - Answers: what exists where?

3. `actors/`
   - Moving or decision-making entities.
   - Player, customers, workers, raiders, travelers, movement, actor state, spawning, patience, path-following, animation state owned by an actor.
   - Answers: who is acting, moving, waiting, leaving, working, or fighting?

4. `gameplay/`
   - Non-visual simulation and business rules.
   - Tavern resources, orders, stations, interactions, jobs, AI decisions, events, factions, inventory, upgrades, reputation, morale, threat, save/load logic for game state.
   - Answers: what rule changes the game state?

5. `ui/`
   - Player-facing interface.
   - HUD, prompts, menus, debug panels, inventory screens, build screens, settings, tooltips.
   - UI may read gameplay/world/actor state and emit player intent, but it should not advance simulation state directly.

6. `rendering/`
   - Shared visual infrastructure.
   - Sprite helpers, mesh builders, materials, visual effects, render adapters, animation presentation helpers, asset-to-render setup.
   - Answers: how does logical game state become visible?
   - Must not own simulation rules, resource changes, AI decisions, or map truth.

## Support Modules

- `config.rs` is for short-lived prototype constants or cross-module tuning values. Prefer moving stable data into domain-owned data files later.
- `mod.rs` files should declare module structure and re-export only the public surface needed by other domains.

## Split Rule

If a new feature seems to belong to more than one domain, split it by responsibility instead of creating a large mixed file.

Example:

- Customer movement: `actors/customer.rs`
- Customer order rules: `gameplay/orders.rs`
- Seat markers and entrances: `world/layout.rs`
- Customer prompt text: `ui/hud.rs`
- Customer sprite sizing helper: `rendering/sprites.rs`

Avoid catch-all files named `utils.rs`, `helpers.rs`, `systems.rs`, or `manager.rs` unless the project has a clear, narrow local pattern for them.

## Asset, Data, And Documentation Placement

- Runtime art assets: `isometric_game/assets/<kind>/`
- Source/reference images and art direction: `isometric_game/data/`
- Design and engineering documents: `isometric_game/docs/`
- Future structured game data: prefer a domain-relevant data folder or clearly named files such as `items.ron`, `stations.ron`, `maps/*.ron`.

When in doubt, choose the folder by asking which domain would break if the file disappeared.
