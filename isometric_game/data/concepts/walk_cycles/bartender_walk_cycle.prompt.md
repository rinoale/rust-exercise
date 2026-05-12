# Bartender Walk Cycle Prompt History

Associated source sheets:

- `bartender_walk_000_guided.png` -> `NorthEast`
- `bartender_walk_045_guided.png` -> `East`
- `bartender_walk_090_guided.png` -> `SouthEast`
- `bartender_walk_135_guided.png` -> `South`
- `bartender_walk_180_guided.png` -> `SouthWest`
- `bartender_walk_225_guided.png` -> `West`
- `bartender_walk_270_guided.png` -> `NorthWest`
- `bartender_walk_315_guided.png` -> `North`

Status: candidate walk-cycle source art, accepted for current runtime prototype with human-in-the-loop tolerance for sprite cleanup.

Generated with: Codex built-in image generation tool, documented as defaulting to `gpt-image-2`.

Prompt pattern:

```text
Use case: stylized-concept
Asset type: auditable six-frame walking animation strip for one direction in a 2.5D isometric Bevy tavern game
Reference image role: use the approved guided 8-direction turnaround template visible in the conversation as the character identity, visual style, camera, scale, crop spacing, and <DEGREE>-degree direction reference.
Primary request: Generate a stop-motion sprite strip of one complete walking cycle for the same wasteland tavern keeper, facing exactly <DEGREE> degrees. The character is walking forward <SCREEN_DIRECTION>. Create exactly 6 animation frames in one horizontal row.
Subject: same tired wasteland tavern keeper from the reference, full body, practical realistic proportions, dirty apron over worn jacket, boots, belt pouches, rolled sleeves, rough survival clothing, same exact character identity, outfit, silhouette, proportions, and gear in every frame.
Style/medium: grounded semi-realistic isometric tactical-survival game sprite art, material-focused, less cartoon, muted brown olive gray sepia rust dirty canvas old leather, weak warm tavern light, dark readable outlines, hand-painted pixel-art-like finish, compact game-sprite readability.
Canvas layout: exactly 6 equal-size vertical cells in one horizontal row. Each cell has one character only. Use generous gutter between cells. Same scale, same foot anchor baseline, same camera height, same lighting, same outfit details in every frame.
Direction requirement: all 6 frames face exactly <DEGREE> degrees only. <FACING_DESCRIPTION>. Do not turn to any other angle.
Animation requirement: show a natural single walking cycle as stop-motion frames. The six frames should progress smoothly: contact pose, passing pose, opposite contact pose, passing pose, contact pose, recovery pose. Feet and arms alternate naturally across the sequence. The character should look like he is walking forward in the <DEGREE>-degree direction, not standing idle. Keep the torso upright and grounded; no running, jumping, lunging, or exaggerated stride.
Audit guides: Put a small black frame label above each cell: F1, F2, F3, F4, F5, F6. Put a red arrow above the row pointing <ARROW_DIRECTION> to show the shared walking direction. Put small text above the whole sheet: WALK CYCLE <DEGREE> DEGREE. Keep all labels and arrows outside the character crop area. The character itself must have no text.
Background: perfectly flat solid #00ff00 chroma-key background for later removal; no shadows, no gradient, no texture, no floor plane, no contact shadow, no reflections.
Hard constraints: exactly six characters total. All six must be <FACING_SHORT>. Do not create an 8-direction sheet. Do not use idle standing poses. Do not duplicate the same pose in every cell. Do not randomly change the character identity or outfit between frames. Do not use #00ff00 in the character. Avoid cute, chibi, clean cartoon, vector-like, toy-like, oversized, glossy, exaggerated shapes, watermark, UI panels, extra props, multiple different characters, or scenery.
```
