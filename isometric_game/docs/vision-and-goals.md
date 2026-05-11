# Vision And Goals

## Working Title

Last Call Tavern

## One Sentence Pitch

Run a battered survivor tavern in an apocalyptic wasteland: feed travelers, manage scarce supplies, keep the room calm, and decide who gets shelter when the world outside keeps getting worse.

## Core Fantasy

The player is not a superhero. They are the person keeping one warm room alive. Every customer is a risk, every meal costs supplies, every broadcast may attract help or danger, and every boarded window says the tavern survived one more night.

## Design Pillars

1. **Tavern First**
   The tavern is the main character. Most systems should make the building feel useful, fragile, lived in, and worth improving.

2. **Scarcity With Choices**
   Food, water, medicine, scrap, time, and trust should remain meaningful. The player should often choose between imperfect options.

3. **Readable 2.5D Survival Space**
   The game should use a fixed isometric view with layered 2D assets. Floors, walls, roofs, props, and characters must remain visually readable during play.

4. **People Bring Stories**
   Customers are not just counters. Survivors should have needs, moods, traits, rumors, risks, and consequences.

5. **Cozy Shelter, Hostile World**
   The tavern should feel warm, dense, furnished, and personal. Cozy atmosphere is a visual and emotional contrast to the wasteland, not a productivity-game feature set.

6. **Build Systems From The Basement**
   We are intentionally using Rust and Bevy. We accept that building the engine layer is part of the project.

## Primary Gameplay Loop

1. Survivors arrive at the tavern.
2. The player evaluates their needs and risk.
3. The player serves food, drink, shelter, information, or refuses service.
4. Resources, morale, reputation, threat, and tavern condition change.
5. The player repairs, cleans, cooks, filters water, trades, and prepares for the next wave.
6. Events from the wasteland change the next set of decisions.

## Major Gameplay Loops

- Tavern management: service, shelter, staff, upgrades, reputation.
- Resource gathering and crafting: wood, scrap, food, water, fuel, tools, furniture, fortifications.
- Defense: raids, barricades, traps, armed workers, layout-driven tactics.
- Exploration and trade: towns, ruins, vehicles, horses, fuel, routes, breakdowns.
- Diplomacy and dilemmas: hostile towns, faction standing, secrets, rumors, moral choices.
- Tavern life: distillery, bounty board, drunk trouble, gossip, entertainment, morale.
- Survival pressure: day/night, seasons, weather, rest, hunger, morale, danger.

## Target Feel

- Quiet tension, not constant combat.
- Warm light inside, hostile world outside.
- Cozy, carefully furnished shelter atmosphere without becoming a productivity companion app.
- Practical survival management with human consequences.
- Gritty hand-painted isometric visuals, not clean cartoon sprites.

## Initial Scope

The first vertical slice should include:

- One playable tavern interior.
- Player movement and interaction.
- Customers entering, waiting, ordering, paying, and leaving.
- Food, water, scrap, morale, reputation, and threat resources.
- Basic stations: bar, stove, pantry, water still, radio, storage.
- Furniture placement or object placement enough to make the tavern feel authored and upgradeable.
- 2.5D rendering with floors, walls, props, characters, Y-sorting, and cutaway rules.
- Save/load for the tavern state.

## Non-Goals For Early Development

- No open-world map yet.
- No complex combat system yet.
- No multiplayer.
- No full procedural building generator.
- No high-end lighting pipeline.
- No focus timer, real-life task board, study-room mode, or Virtual Cottage-style productivity system in the core plan.
- No exact imitation of any existing game asset style or IP.

## Long-Term Goals

- Multiple rooms and tavern upgrades.
- Hireable workers with traits and jobs.
- Distillery, bounty board, clinic, workshop, radio tower, cellar, stable, watchtower, farm, and garage upgrades.
- Day/night cycle and nightly threat events.
- Survivor traits, factions, town politics, and reputation.
- Trade, smuggling, rumors, bounty jobs, caravans, vehicles, horses, and route risk.
- Farming, animals, hunting, scavenging, and crafting.
- Repairable walls, doors, windows, and barricades.
- Simple combat or defense events if they serve tavern management.
- Data-driven content files for items, stations, events, rooms, and NPC archetypes.
