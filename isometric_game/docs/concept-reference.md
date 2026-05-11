# Concept Reference

This document distills the design intent from the earlier Unity project note at `/mnt/c/Users/Admin/Documents/Unity/apocalypse_tavern/CLAUDE.md`.

## Core Concept

Last Call Tavern is a 2D isometric tavern management simulation set in a post-apocalyptic world. The player runs a lone tavern far from towns and residences, where travelers, workers, smugglers, raiders, and faction agents pass through.

The tavern is neutral ground, but neutrality is fragile. The player survives by serving people, managing supplies, upgrading the building, gathering information, and choosing how involved to become in the politics of hostile towns.

## Main Gameplay Areas

### Tavern Management

- Serve travelers food, drinks, shelter, and information.
- Hire workers with useful traits: cook, mechanic, medic, scout, guard, lumberjack.
- Upgrade and expand the tavern with crafted furniture, structures, and rooms.
- Maintain reputation through service quality and decisions.

### Resource Gathering And Crafting

- Gather wood, scrap metal, electronics, food, water, fuel, and medicine.
- Farm crops and eventually raise animals.
- Hunt, scavenge, and loot abandoned locations.
- Craft furniture, weapons, tools, drinks, food, and fortifications.

### Combat And Defense

- Defend against scavenger raids and hostile visitors.
- Build barricades, traps, reinforced walls, and watch points.
- Arm workers or hire protection.
- Use isometric placement and building layout as tactical defense tools.

### Exploration And Trade

- Travel between ruined towns and locations.
- Use vehicles, horses, or caravans depending on upgrades and resources.
- Trade and smuggle goods between hostile towns.
- Manage fuel, repairs, travel risk, and route information.

### Diplomacy And Dilemmas

- Towns and factions are hostile to each other.
- The tavern can remain neutral, choose sides, or play both sides.
- Travelers bring rumors, jobs, secrets, and trouble.
- Information is a currency: overhear it, sell it, hide it, or act on it.

### Reputation And Relationships

- Traveler satisfaction affects word-of-mouth.
- Good reputation can attract better customers, rare goods, protection, and trade information.
- Bad reputation can attract raiders, bounty hunters, poor deals, and fewer visitors.
- Faction standing affects prices, access, threats, and quests.

### Tavern Life

- Distillery for signature apocalypse drinks and moonshine.
- Bounty board for fetch, rescue, assassination, delivery, escort, and intel jobs.
- Drunk customers can cause fights, break furniture, or reveal secrets.
- Music and entertainment can improve morale and atmosphere.
- Gossip and overheard conversations can become actionable intel.

## Upgrade Directions

- Stable: horse travel without fuel.
- Clinic: healing service, medic worker, injured traveler gameplay.
- Workshop: repair and craft better gear.
- Radio tower: raid warnings, trade routes, faction news, long-range rumors.
- Cellar: storage, preservation, and aging drinks.
- Watchtower: early warning and defense bonuses.
- Farm/yard: crops, animals, water collection, wood storage.
- Garage: vehicle repair and travel expansion.

## Survival Pressure

- Day/night cycle changes danger and customer flow.
- Seasons affect farming, travel, and resource needs.
- Weather affects scavenging, travel, comfort, and raids.
- The player character and workers may need rest, food, safety, and morale.

## Atmosphere Reference

The cozy reference is about shelter, not productivity mechanics.

Use cozy atmosphere for:

- Warm lighting against a bleak outside world.
- Carefully placed furniture and props.
- A room that becomes more personal as the tavern improves.
- Interior clutter that tells survival stories.
- Calm pockets between danger and scarcity.

Do not import cozy-productivity systems by default:

- No focus timer as a core feature.
- No real-life task board as a core feature.
- No study-room multiplayer as a core feature.
- No ambient control panel as a primary game system.

If any of those are considered later, they should support the tavern fantasy rather than redefine the game.
