# Dev Playback

Dev playback scripts let the game drive itself when direct keyboard automation is unavailable.

Current script:

```bash
cargo run --bin isometric_game -- --script smoke_walk_and_serve
```

What it does:

- Opens the normal Bevy game window.
- Enables grid, collision, and depth debug overlays.
- Moves the player to pantry, water still, stove, and radio.
- Uses each station through the shared interaction logic.
- Waits for a customer to sit down.
- Moves to that customer and serves them.
- Prints initial and final tavern resource state.
- Exits the app automatically.

This is not a replacement for human playtesting. It is a smoke test for basic runtime wiring: world spawn, player movement by grid position, interaction reach, station actions, customer serving, debug overlays, and app exit.

Future scripts should stay narrow and named by behavior, for example:

```text
smoke_walk_and_serve
smoke_cutaway_and_debug
smoke_resource_pressure
smoke_customer_patience
```
