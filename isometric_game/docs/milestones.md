# Milestones

This roadmap assumes Rust and Bevy remain the foundation. Each milestone should end with a playable build, even if art and content are still rough.

## Milestone 0: Prototype Baseline

Status: In progress.

Goal: Preserve the current playable loop while replacing placeholder visuals with project art direction.

Deliverables:

- Bevy project runs with player movement, camera follow, and zoom.
- Tavern resources exist: food, water, scrap, morale, reputation, threat.
- Customers can enter, wait, be served, and leave.
- Stations can be interacted with.
- Art direction prompts and concept images are stored in the repo.
- First runtime character and tile sprites are loaded by Bevy.
- Docs clearly state that cozy references are atmosphere-only, while systems follow the apocalypse tavern plan.

Done when:

- `cargo check` passes.
- `cargo run` starts a playable tavern prototype.
- The repo contains clear documentation for goals and next milestones.

## Milestone 1: 2.5D Isometric Foundation

Goal: Build the rendering and world model needed for a Project Zomboid-like viewpoint without copying its assets or systems.

Deliverables:

- `GridPos` and world-coordinate model separated from render transforms.
- Tile layers: ground, floor, wall, roof, prop, character, overlay.
- Consistent sprite anchors for tiles, props, walls, and characters.
- Y-sorted draw ordering for characters and props.
- Collision and walkability grid.
- Basic object occlusion: character can pass behind tables, bar, and walls.
- Debug overlay for grid cells, collision, and draw order.

Done when:

- A character can walk around a furnished tavern without visual depth errors in normal cases.
- Props and characters sort predictably by grid position.
- Walkable and blocked cells are visible in a debug mode.

## Milestone 2: Building Asset Pipeline

Goal: Define how building pieces are created, named, imported, anchored, and assembled.

Deliverables:

- Asset naming convention for floors, walls, corners, doors, windows, props, roofs, and shadows.
- Sprite size and anchor rules documented.
- Texture atlas plan for runtime sprites.
- First wall and floor kit for the tavern interior.
- Roof/cutaway rules for showing interiors.
- Data file format for placing building pieces.

Done when:

- A tavern room can be rebuilt from data instead of hardcoded entity placement.
- Art assets align to the grid without per-asset manual fixes.
- Roof or upper wall visibility changes when the player is inside.

## Milestone 3: Tavern Interaction Loop

Goal: Make tavern management feel intentional rather than a collection of test interactions.

Deliverables:

- Interactable station component with data-driven actions.
- Inventory/resource costs and outputs.
- Customer orders with clear requirements and rewards.
- Tavern services beyond meals: drink, shelter, rumors, basic trade.
- Clean prompt system for nearby actions.
- Station cooldowns or task durations.
- Consequences for ignoring customers, filth, and shortages.

Done when:

- The player can run a 5-minute session with meaningful resource changes.
- At least three different customer needs require different station usage.
- Tavern actions change reputation, morale, threat, and supply pressure.
- The HUD explains current pressure without becoming noisy.

## Milestone 4: NPC Behavior And Pathfinding

Goal: Replace simple target movement with robust tavern navigation and character state.

Deliverables:

- Grid pathfinding across walkable cells.
- Seats, queue points, work points, and exits as named markers.
- Customer states: entering, waiting, ordering, eating/drinking, paying, leaving, angry.
- NPC patience, traits, and risk tags.
- Worker NPCs with early job roles: cook, guard, medic, mechanic, scout.
- Staff/player task interruption rules.

Done when:

- Multiple NPCs can enter, find seats, avoid blocked cells, and leave without overlapping constantly.
- NPC state can be inspected in debug UI.
- Bad service creates visible behavior changes.

## Milestone 5: Save, Load, And Data-Driven Content

Goal: Make the game state persistent and content expandable.

Deliverables:

- Serializable tavern state.
- Serializable map/building layout.
- Data files for items, stations, customer archetypes, and events.
- Data files for worker roles, faction/town standing, jobs, and rumors.
- Save/load menu or hotkeys.
- Versioned save format notes.

Done when:

- The player can quit and resume a tavern state.
- Adding a new item or station does not require editing core gameplay code.

## Milestone 6: Vertical Slice

Goal: Produce one coherent small game slice that proves the project is viable in Bevy.

Deliverables:

- One polished tavern interior.
- One exterior or entry area.
- Full 2.5D sorting and cutaway behavior for that space.
- Ten customer archetypes.
- Ten events or rumors.
- At least one upgrade choice.
- One small faction/reputation dilemma.
- One defense or threat event that uses tavern layout.
- Music or ambient sound pass.
- Basic start, pause, and save/load flow.

Done when:

- A new player can understand the goal without reading developer notes.
- A 15-minute session creates different outcomes based on choices.
- The game looks visually aligned with the stored art direction.

## Milestone 7: Wider Apocalypse Systems

Goal: Expand from a tavern-room slice into the broader concept without losing the tavern as the center.

Deliverables:

- Distillery and signature drink quality.
- Bounty board with simple job generation.
- First worker hiring and assignment flow.
- First trade route or scavenging destination.
- Faction standing for at least two towns or groups.
- Upgrade chain for one major room: clinic, workshop, cellar, stable, radio tower, or watchtower.
- Basic raid/defense loop with barricades or traps.

Done when:

- The tavern connects to the world through trade, rumors, jobs, and faction consequences.
- The player can prepare for danger by changing the tavern layout or upgrades.
- Expansion systems still feed back into tavern service and survival pressure.

## Milestone 8: Production Decision

Goal: Decide whether to continue scaling the custom Bevy stack or change direction.

Evaluation questions:

- Is the 2.5D foundation stable enough?
- Can we add content without fighting the tooling?
- Is Bevy performance acceptable with expected sprite counts?
- Are save/load and data workflows sane?
- Is art integration repeatable?

Decision:

- Continue in Bevy if the vertical slice is stable and content authoring is manageable.
- Rework tooling if the gameplay is strong but content placement is too slow.
- Reconsider engine only if the custom foundation blocks basic production.
