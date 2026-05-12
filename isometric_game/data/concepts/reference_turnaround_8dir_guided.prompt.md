# Reference Turnaround 8-Direction Guided Prompt

Associated image: `reference_turnaround_8dir_guided.png`

Status: human-approved template prompt for generating auditable 8-direction character turnaround candidates.

Generated with: built-in image generation tool.

```text
Use case: stylized-concept
Asset type: auditable production character rotation sprite-sheet candidate for a 2.5D isometric Bevy tavern game
Primary request: Generate one original apocalyptic tavern survivor character turnaround sheet with enforced degree guides. The goal is correct visible rotation order, not a pretty concept sheet.
Subject: one tired wasteland tavern keeper, full body, practical realistic proportions, dirty apron over worn jacket, boots, belt pouches, rolled sleeves, rough survival clothing, same exact outfit and silhouette in every pose.
Style/medium: grounded semi-realistic isometric tactical-survival game sprite art, material-focused, less cartoon, muted brown olive gray sepia rust dirty canvas old leather, weak warm tavern light, dark readable outlines, hand-painted pixel-art-like finish, compact game-sprite readability.
Canvas layout: exactly 8 equal-size vertical cells in one horizontal row. Each cell has one character only. Use generous gutter between cells. Same character, same scale, same foot position, same camera height, same lighting, same outfit details in every cell.
Critical rotation order: The character must turn clockwise as the columns progress from left to right. Column 1 is 0 degrees and shows the back of the character facing screen-up / away from viewer. Column 2 is 45 degrees clockwise, back-right three-quarter. Column 3 is 90 degrees clockwise and must be a right-facing side profile, the character's face looking toward screen-right. Column 4 is 135 degrees clockwise, front-right three-quarter. Column 5 is 180 degrees and shows the front of the character facing screen-down / toward viewer. Column 6 is 225 degrees clockwise, front-left three-quarter. Column 7 is 270 degrees clockwise and must be a left-facing side profile, the character's face looking toward screen-left. Column 8 is 315 degrees clockwise, back-left three-quarter.
Audit guides: Put a small black degree label above each cell: 0, 45, 90, 135, 180, 225, 270, 315. Put a small red arrow above each label showing the intended facing direction on the screen. Keep labels and arrows outside the character crop area. The character itself must have no text.
Background: perfectly flat solid #00ff00 chroma-key background for later removal; no shadows, no gradient, no texture, no floor plane, no contact shadow, no reflections.
Hard constraints: Do not swap 90 and 270. Column 3 must face screen-right. Column 7 must face screen-left. Do not swap 135 and 225. Column 4 must be front-right three-quarter. Column 6 must be front-left three-quarter. Do not duplicate poses. Do not mirror clothing details randomly between columns. Do not use #00ff00 in the character. Avoid cute, chibi, clean cartoon, vector-like, toy-like, oversized, glossy, exaggerated shapes, watermark, UI panels, extra props, multiple characters, or scenery.
```
