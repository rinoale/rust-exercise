# Milestones

This roadmap assumes Rust and Bevy remain the foundation. Each milestone should end with a playable build, even if art and content are still rough.

## Tracking Rule

- `[x]` means implemented or documented in the current repo.
- `[ ]` means not implemented, not verified, or still too rough to count.
- Keep this file updated whenever a milestone item becomes real.

## Milestone 0: Prototype Baseline

Status: Complete for the baseline. User screenshots confirm the windowed prototype launches; this assistant's WSL session still cannot open a display because of `XOpenDisplayFailed`.

Goal: Preserve the current playable loop while replacing placeholder visuals with project art direction.

Deliverables:

- [x] Bevy project has player movement, camera follow, and zoom systems.
- [x] Tavern resources exist: food, water, scrap, morale, reputation, threat.
- [x] Customers can enter, wait, be served, lose patience, and leave.
- [x] Stations can be interacted with.
- [x] Art direction prompts and concept images are stored in the repo.
- [x] First runtime character and tile sprites are loaded by Bevy.
- [x] Docs clearly state that cozy references are atmosphere-only, while systems follow the apocalypse tavern plan.
- [x] Source code is split into the six core domains: universe, world, actors, gameplay, ui, rendering.

Done when:

- [x] `cargo check` passes.
- [x] `cargo run` starts a playable tavern prototype.
- [x] The repo contains clear documentation for goals and next milestones.

## Milestone 1: 2.5D Isometric Foundation

Status: Complete for the first foundation pass. User screenshots confirm normal-case walkaround, grid, collision, and depth debug visualization.

Goal: Build the rendering and world model needed for a Project Zomboid-like viewpoint without copying its assets or systems.

Deliverables:

- [x] Grid position model separated from render transforms.
- [x] Grid position and world collision support discrete `z` levels for stacked floors/walls.
- [x] Tile layers include the full target set: ground, floor, decal, wall, roof, prop, character, overlay.
- [x] Consistent sprite anchors for tiles, props, walls, and characters.
- [x] Y-sorted draw ordering for characters and props.
- [x] Collision and walkability grid.
- [x] Shared universe kinematic movement primitive used by player and NPCs.
- [x] Basic object occlusion rules: shared anchors, layer offsets, and Y-sort tests cover props, characters, and walls.
- [x] Debug overlay for grid cells, collision, and depth anchors.
- [x] World placement comes from a reusable blueprint instead of scattered hardcoded spawns.

Done when:

- [x] A character can walk around a furnished tavern without visual depth errors in normal cases.
- [x] Props and characters sort predictably by grid position.
- [x] Walkable and blocked cells are visible in a debug mode.

## Milestone 2: Building Asset Pipeline

Status: Complete for the first data-driven building pass. The demo world no longer needs hand-tuned wall/roof offsets.

Goal: Define how building pieces are created, named, imported, anchored, and assembled.

Deliverables:

- [x] Asset naming convention for floors, walls, corners, doors, windows, props, roofs, and shadows.
- [x] Sprite size and anchor rules documented.
- [x] Texture atlas plan for runtime sprites.
- [x] First wall and floor kit for the tavern interior.
- [x] Roof/cutaway rules for showing interiors.
- [x] Data file format for placing building pieces.
- [x] Reusable prop style presets for world data files.

Done when:

- [x] A tavern room can be rebuilt from data instead of hardcoded entity placement.
- [x] Art assets align to the grid without per-asset manual fixes.
- [x] Roof or upper wall visibility changes when the player is inside.

## Milestone 3: Tavern Interaction Loop

Status: Complete for the first interaction-loop pass. User screenshots confirm a 5+ minute graphical run with meaningful resource changes.

Goal: Make tavern management feel intentional rather than a collection of test interactions.

Deliverables:

- [x] Station component exists.
- [x] Station actions are data-driven.
- [x] Inventory/resource costs and outputs.
- [x] Customer orders with clear requirements and rewards.
- [x] Tavern services beyond meals and drinks: shelter, rumors, basic trade.
- [x] Clean prompt system for nearby actions.
- [x] Station cooldowns or task durations.
- [x] Consequences for ignoring customers, filth, and shortages.

Done when:

- [x] The player can run a 5-minute session with meaningful resource changes.
- [x] Automated 5-minute service-loop smoke test has meaningful resource changes.
- [x] At least three different customer needs require different station usage.
- [x] Tavern actions change reputation, morale, threat, and supply pressure.
- [x] The HUD explains current pressure without becoming noisy.

## Milestone 4: Game-Ready Sprite And Animation Foundation

Status: Started. Shared actor visual-motion state, 8-direction facing resolution, clip metadata loading, frame selection, guided-turnaround import, and 8-direction six-frame player walk cycles are in place.

Goal: Replace static character sprites with a real sprite/animation pipeline that can use 8-direction character sheets and future generated assets.

Deliverables:

- [x] Runtime animation components for direction, animation state, frame timing, and sprite frame selection.
- [x] Asset metadata format for character clips: idle, walk, service, and hurt/angry placeholders.
- [x] Main character uses validated 8-direction idle and walk sprites.
- [ ] At least three NPC archetypes use direction-aware idle and walk sprites.
- [x] Movement direction drives facing direction without hardcoded per-character logic.
- [x] Animation debug view or debug text shows current character state, direction, and clip.
- [x] Sprite extraction plan for existing concept sheets, including transparent-background runtime output.

Done when:

- [ ] The player visibly animates while moving in all 8 directions.
- [ ] NPCs no longer look like static cutouts sliding across the floor.
- [ ] Adding a new animated character does not require changing movement or rendering systems.
- [x] `cargo check` and animation-related tests pass.

## Milestone 5: Tavern Environment Art Pass

Status: Started. First concept-derived runtime sprites are extracted and the demo tavern can use sprite-backed props, walls, roof, stations, and tile variants.

Goal: Replace the simple prototype room with art-directed 2.5D tavern assets: real tiles, opaque walls, roof pieces, props, shadows, and decoration.

Deliverables:

- [x] Runtime tile set with at least three worn floor variants and two wasteland ground variants.
- [x] Wall sprite kit built from the concept art direction: back wall, side wall, corners, door, window, posts, trim, and damaged variants.
- [x] Roof sprite kit with cutaway-compatible repeated pieces instead of one stretched full-building cover.
- [x] First real prop kit: bar counter, table, stool, stove, pantry, water still, radio, crates, barricades, lamp, shelf, and rug.
- [ ] Decoration/decal set: stains, dirt, cracks, papers, bottle clutter, floor wear, soot, and warm light pools.
- [x] World RON references visual asset ids instead of prototype mesh colors for normal tavern objects.
- [x] Art fallback rule: if existing concept sheets do not contain a needed piece, create or generate a matching asset using `data/art_direction.md`.

Done when:

- [ ] Screenshots no longer read as colored blocks, transparent wall placeholders, or flat prototype objects.
- [ ] Walls are opaque readable building pieces while still supporting interior cutaway.
- [ ] Tiles and props visually match the stored art direction.
- [ ] Collision-bearing props use isolated runtime sprites instead of grouped concept-art clusters.
- [x] The tavern still rebuilds from data and remains playable.

## Milestone 6: Visual Integration And Game Feel Pass

Status: Started. Actors have simple idle/walk visual motion; richer interaction and station feedback are still pending.

Goal: Make the upgraded art feel alive in play rather than only looking correct in static screenshots.

Deliverables:

- [x] Idle fidgets or subtle breathing for player and common NPCs.
- [ ] Service interaction animation or pose feedback when serving guests and using stations.
- [ ] Station feedback: stove warmth, radio signal pulse, water still activity, pantry use, and cooldown state.
- [ ] Prop interaction feedback for cleaning spills and serving customers.
- [ ] Camera, zoom, and UI readability pass against the denser art.
- [ ] Screenshot comparison checklist against `data/concepts/last_call_interior.png`, `data/environments.png`, and `data/characters.png`.

Done when:

- [ ] A 5-minute session looks like a small game scene, not a systems prototype.
- [ ] Player, NPCs, props, walls, roof, and tiles share one coherent visual style.
- [ ] Visual feedback makes interaction outcomes readable without relying only on HUD text.

## Milestone 7: NPC Behavior And Pathfinding

Status: Not started.

Goal: Replace simple target movement with robust tavern navigation and character state.

Deliverables:

- [ ] Grid pathfinding across walkable cells.
- [ ] Seats, queue points, work points, and exits as named markers.
- [ ] Customer states: entering, waiting, ordering, eating/drinking, paying, leaving, angry.
- [ ] NPC patience, traits, and risk tags.
- [ ] Worker NPCs with early job roles: cook, guard, medic, mechanic, scout.
- [ ] Staff/player task interruption rules.

Done when:

- [ ] Multiple NPCs can enter, find seats, avoid blocked cells, and leave without overlapping constantly.
- [ ] NPC state can be inspected in debug UI.
- [ ] Bad service creates visible behavior changes.

## Milestone 8: In-Game Scene Editor And Content Authoring

Status: Started. Stable asset IDs, prefab/archetype data, explicit world launch, editor mode, scene selector, editor placement tools, save, and manual hot reload are in place. Richer validation, metadata inspector editing, undo/redo, and full editor polish remain.

Goal: Let scenes, object placement, collision footprints, station markers, and visual tuning change through data files without Rust source edits or rebuilds.

Principle: Rust owns engine behavior. World `.ron` files and editor tools own content.

Deliverables:

- [x] Asset registry with stable sprite IDs, asset kinds, path validation, and missing-file errors.
- [x] Prefab/archetype data format for reusable world objects and station-ready props.
- [x] Launch the game with an explicit world path, for example `--world data/worlds/demo_tavern.ron`.
- [x] Editor mode toggle that does not break normal play mode.
- [x] Editor startup scene selector with a zero-base empty scene option.
- [x] Grid cursor with `x/y/z` coordinate readout.
- [x] Mouse-first editor toolbar with visible tool/action buttons and scene click placement/selection.
- [x] Palette for prefab IDs backed by registered visual asset IDs.
- [x] Place, move, duplicate, and delete prefab-backed world objects from the editor.
- [x] World data supports z-aware navigation markers, vertical links, edge walls, and floor visibility filtering.
- [x] Editor tools for tile painting, collision cells, edge walls, markers, vertical links, zones, and world level count.
- [x] Empty scene creation with no default ground tiles.
- [ ] Edit per-object basics: grid position, layer, anchor preset, render size, depth offset, collision footprint, station id, cutaway zone, and marker id.
- [x] Save the edited world back to a `.ron` file.
- [x] Reload the current world file without recompiling.
- [x] Hot reload the asset registry, prefab files, and current world after validation succeeds.
- [x] Validation report before save for load/build errors such as missing assets, invalid z values, invalid links, and malformed world data.
- [ ] Rich semantic validation: overlapping blocking objects, station without gameplay definition, marker id duplicates.
- [ ] Undo/redo for recent editor actions.
- [ ] Editor-only debug overlays for collision, anchors, depth sort order, cutaway zones, and station interaction radius.
- [x] Documentation for authoring a new scene from scratch.

Done when:

- [x] A new room or exterior scene can be created by running the editor, not by editing Rust code.
- [x] Changing object position, collision, markers, edge walls, tiles, zones, or vertical links does not require `cargo build`.
- [ ] The saved `.ron` can be loaded by normal play mode.
- [ ] A content mistake gives a readable validation error instead of a silent broken scene.

## Milestone 9: Save, Load, And Data-Driven Content

Status: Not started.

Goal: Make the game state persistent and content expandable.

Deliverables:

- [ ] Serializable tavern state.
- [ ] Serializable map/building layout.
- [ ] Data files for items, stations, customer archetypes, and events.
- [ ] Data files for worker roles, faction/town standing, jobs, and rumors.
- [ ] Save/load menu or hotkeys.
- [ ] Versioned save format notes.

Done when:

- [ ] The player can quit and resume a tavern state.
- [ ] Adding a new item or station does not require editing core gameplay code.

## Milestone 10: Vertical Slice

Status: Not started.

Goal: Produce one coherent small game slice that proves the project is viable in Bevy.

Deliverables:

- [ ] One polished tavern interior.
- [ ] One exterior or entry area.
- [ ] Full 2.5D sorting and cutaway behavior for that space.
- [ ] Ten customer archetypes.
- [ ] Ten events or rumors.
- [ ] At least one upgrade choice.
- [ ] One small faction/reputation dilemma.
- [ ] One defense or threat event that uses tavern layout.
- [ ] Music or ambient sound pass.
- [ ] Basic start, pause, and save/load flow.

Done when:

- [ ] A new player can understand the goal without reading developer notes.
- [ ] A 15-minute session creates different outcomes based on choices.
- [ ] The game looks visually aligned with the stored art direction.

## Milestone 11: Wider Apocalypse Systems

Status: Not started.

Goal: Expand from a tavern-room slice into the broader concept without losing the tavern as the center.

Deliverables:

- [ ] Distillery and signature drink quality.
- [ ] Bounty board with simple job generation.
- [ ] First worker hiring and assignment flow.
- [ ] First trade route or scavenging destination.
- [ ] Faction standing for at least two towns or groups.
- [ ] Upgrade chain for one major room: clinic, workshop, cellar, stable, radio tower, or watchtower.
- [ ] Basic raid/defense loop with barricades or traps.

Done when:

- [ ] The tavern connects to the world through trade, rumors, jobs, and faction consequences.
- [ ] The player can prepare for danger by changing the tavern layout or upgrades.
- [ ] Expansion systems still feed back into tavern service and survival pressure.

## Milestone 12: Production Decision

Status: Future decision.

Goal: Decide whether to continue scaling the custom Bevy stack or change direction.

Evaluation questions:

- [ ] Is the 2.5D foundation stable enough?
- [ ] Can we add content without fighting the tooling?
- [ ] Is Bevy performance acceptable with expected sprite counts?
- [ ] Are save/load and data workflows sane?
- [ ] Is art integration repeatable?

Decision:

- Continue in Bevy if the vertical slice is stable and content authoring is manageable.
- Rework tooling if the gameplay is strong but content placement is too slow.
- Reconsider engine only if the custom foundation blocks basic production.
