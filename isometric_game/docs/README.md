# Last Call Tavern Documentation

This folder tracks the planning documents for the Rust and Bevy 2.5D isometric tavern game.

## Documents

- [Vision And Goals](vision-and-goals.md): game identity, pillars, scope, and non-goals.
- [Concept Reference](concept-reference.md): distilled design reference from the earlier Unity project note.
- [Milestones](milestones.md): staged roadmap and progress checklist with acceptance criteria.
- [Technical Foundation](technical-foundation.md): Bevy architecture plan for the isometric 2.5D foundation.
- [Asset Guidelines](asset-guidelines.md): naming, anchor, size, collision, and depth rules for 2.5D assets.
- [Texture Atlas Plan](texture-atlas-plan.md): runtime atlas grouping, metadata, packing, and Bevy import direction.
- [Visual Upgrade Plan](visual-upgrade-plan.md): sprite, animation, building art, prop, and visual feedback milestones.
- [Editor Authoring Plan](editor-authoring-plan.md): in-game scene editor scope, data ownership, validation, and authoring workflow.
- [Dev Playback](dev-playback.md): scripted runtime smoke tests for movement, interactions, and visual debug wiring.

## Current Direction

The game is a survivor-run tavern management game in an apocalyptic world. It uses a fixed isometric 2.5D presentation: 2D sprites, layered buildings, grid/world simulation, Y-sorted depth, and hand-painted gritty assets.

Virtual Cottage-like influence is limited to atmosphere: cozy object placement, warm interior mood, furniture density, and the feeling of a safe shelter. It is not a reference for focus timers, productivity task boards, real-time study rooms, or similar systems.

Art direction prompts and retained concept images live in [`../data/art_direction.md`](../data/art_direction.md).
