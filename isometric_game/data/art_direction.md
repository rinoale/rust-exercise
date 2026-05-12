# Last Call Tavern Art Direction

These prompts are the concept-art director for the apocalyptic isometric tavern game. Keep new game assets aligned with these constraints unless intentionally changing the art direction.

## Realistic Tactical Isometric Style Correction

Use this direction for production-facing concept sheets and sprite candidates.

```text
Use a grounded semi-realistic isometric tactical-survival look. The visual target is more realistic and material-focused than cartoon: practical proportions, subdued faces, dirty worn clothing, rough timber, rusted metal, chipped enamel, soot, dust, grime, and readable survival gear.

The spirit can reference classic isometric tactical games and survival RPGs such as Project Zomboid or Commandos at a high level, but do not copy their assets, characters, UI, palette, or exact rendering style.

Avoid cute, chibi, clean cartoon, vector-like, toy-like, oversized, glossy, or exaggerated shapes. Keep the palette muted: brown, olive, gray, sepia, rust, dirty canvas, old leather, and weak warm tavern light.

Concept sheets should keep objects separated on a dark neutral background so they can later be selected, cleaned up, sliced, and turned into runtime sprites.
```

## Runtime Sprite Sheet Addendum

Use this addendum when the output is meant to become Bevy runtime sprites, not only mood concept art.

```text
Generate a production sprite sheet for extraction. Every object must be isolated in its own cell with at least 12 pixels of empty gutter around the silhouette. Use a transparent background if supported; otherwise use one flat pure matte color that is very different from the asset colors. Do not use textured sheet backgrounds.

Do not let objects overlap. Do not bake separate collision objects into one sprite: a bar counter must not include stools, chairs, customers, or loose props unless the whole group is meant to share one gameplay collision footprint.

Keep all assets on the same fixed 2:1 isometric camera angle. The base of each prop must sit on the same ground-plane axis as the floor diamonds. Long objects such as counters, benches, walls, and shelves must have versions for the correct isometric axis instead of being arbitrary perspective illustrations.

For roofs, generate modular roof tiles and edge/corner pieces at native runtime resolution. Do not generate one full-building roof that must be stretched across the building.

For extraction safety, add a clean alpha boundary around each silhouette. If transparent output is unavailable, place a temporary high-contrast matte outline outside the object boundary; it is only for slicing and should not be treated as final art.
```

## Character Rotation Sheet Addendum

Use this addendum for playable or NPC character sheets. Do not ask the generator for north, east, south, or west poses. Those words are ambiguous in an isometric project and have produced incorrect labels before.

This prompt creates candidate art only. It is not authority for runtime direction names. A generated sheet can still swap, duplicate, or invent rotations even when the written prompt is clear. Every rotation sheet must be visually audited or rendered from a deterministic 3D/rigged source before it is imported as canonical game animation.

```text
Generate one production character rotation sheet showing the same character turning in place through a full 360 degree rotation. Use exactly 8 columns in one row. The camera angle, scale, lighting, outfit, body proportions, and foot position must remain consistent in every column.

Column 1: 0 degrees, facing straight toward screen-up / away from the viewer.
Column 2: 45 degrees clockwise from column 1.
Column 3: 90 degrees clockwise from column 1, facing screen-right.
Column 4: 135 degrees clockwise from column 1.
Column 5: 180 degrees, facing screen-down / toward the viewer.
Column 6: 225 degrees clockwise from column 1.
Column 7: 270 degrees clockwise from column 1, facing screen-left.
Column 8: 315 degrees clockwise from column 1.

Use transparent background if supported. Otherwise use one flat high-contrast matte color. Put each pose in an equal-size cell with visible empty gutter around the silhouette. Do not write north, east, south, west, northeast, southeast, southwest, or northwest on the image. If temporary production guides are needed, put small degree labels and heading arrows outside the character crop area only, so they can be removed during slicing. Do not duplicate poses. Do not mirror clothing details incorrectly between columns.
```

Runtime mapping from these angle columns to project directions is documented in `docs/technical-foundation.md`.

The first human-approved guided turnaround template and its exact generation prompt are stored as sidecar files:

- `data/concepts/reference_turnaround_8dir_guided.png`
- `data/concepts/reference_turnaround_8dir_guided.prompt.md`

## Exterior Tavern Concept

```text
Use case: stylized-concept
Asset type: game art preview sample
Input images: the local reference images shown in conversation, `characters.png` and `environments.png`, are style references only.
Primary request: create one isometric apocalyptic tavern exterior sample matching the reference art direction.
Scene/backdrop: a dusty post-apocalyptic roadside tavern built from patched wood, corrugated metal, salvaged signs, sandbags, barrels, dead scrub, and cracked dirt; no modern clean surfaces.
Subject: a small survivor-run tavern named "LAST CALL" with an angled isometric view, visible porch, battered windows, makeshift barricades, and warm light leaking from inside.
Style/medium: gritty hand-painted pixel-art-like isometric game concept, muted brown/sepia palette, dense tiny details, dark ink outlines, weathered textures, low-saturation survival atmosphere.
Composition/framing: three-quarter isometric view, full building visible, game asset presentation on a dusty ground plane, similar scale and camera angle to the environment reference.
Lighting/mood: overcast wasteland daylight with a few warm tavern lights, bleak but inhabited.
Materials/textures: rusted sheet metal, rough timber, peeling signs, dirty glass, patched tarps, dry weeds, rubble, tire stacks.
Text (verbatim): "LAST CALL"
Constraints: original design inspired by the provided references; no direct copy of existing reference buildings; no photorealism; no clean cartoon style; no watermark.
```

## Interior Tavern Concept

```text
Use case: stylized-concept
Asset type: game art preview sample
Input images: the local reference images shown in conversation, `characters.png` and `environments.png`, are style references only.
Primary request: create one isometric interior tavern-room sample matching the reference art direction.
Scene/backdrop: inside a survivor tavern after collapse, with wooden plank floor, patched walls, long bar counter, shelves of jars and bottles, cast iron stove, card table, crates, hanging lanterns, warning posters, dirty rugs, scattered tools, and a small pantry corner.
Subject: a playable isometric room layout suitable for a tavern management game; no UI, no labels except worn wall signs if necessary.
Style/medium: gritty hand-painted pixel-art-like isometric game concept, muted sepia/brown palette, black ink detail, dense weathered textures, readable small props, similar to the environment reference interiors.
Composition/framing: cutaway isometric room on a dark neutral void, three-quarter angle, full room visible with stations clearly readable: bar, stove, pantry, seating, storage.
Lighting/mood: dim amber lantern lighting with soft pools of warmth against dark corners, survival shelter mood.
Materials/textures: aged boards, rusted metal, cracked leather, dusty bottles, chipped paint, soot stains, patched cloth, grime.
Constraints: original room design inspired by provided references; no direct copying; no photorealism; no clean vector look; no watermark; no modern polished furniture.
```

## Survivor Sprite Concept Sheet

```text
Use case: stylized-concept
Asset type: game character sprite concept sheet preview
Input images: the local reference images shown in conversation, `characters.png` and `environments.png`, are style references only.
Primary request: create a small character concept sheet for apocalyptic tavern survivors matching the reference character art direction.
Subject: six original survivor/tavern NPC sprites: tired bartender with apron, scavenger customer with backpack, wounded traveler, guarded raider patron, mechanic cook, courier with satchel. Each character shown as small full-body isometric game sprite with consistent proportions.
Style/medium: gritty hand-painted pixel-art-like isometric sprite sheet, muted brown/olive/gray palette, dark outlines, tiny clothing folds, worn fabric, boots, backpacks, survival gear; similar scale and rendering to the character reference.
Composition/framing: arranged in two rows of three on a flat rough brown background, evenly spaced, no UI, no direction labels, each sprite facing south-east three-quarter view.
Lighting/mood: subdued neutral lighting, bleak survival tone.
Materials/textures: dirty coats, patched denim, leather belts, canvas packs, rusted tools, bandages, aprons, worn boots.
Constraints: original characters inspired by the provided references; no direct copying of existing reference characters; no photorealism; no cartoon chibi; no watermark; avoid readable text.
```
