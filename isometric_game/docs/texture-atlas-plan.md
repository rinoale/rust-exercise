# Texture Atlas Plan

This plan keeps early sprites easy to replace while giving the project a stable path toward production atlases.

## Prototype Rule

Individual PNG files are acceptable while the world model, anchors, and gameplay loop are still changing. Do not bake gameplay meaning into an atlas frame name. Placement, collision, anchors, layer, and cutaway behavior stay in RON data or Rust metadata.

## Atlas Groups

Use separate atlases by update frequency and art role:

- `tiles_atlas.png`: ground, floor, porch, road, and repeatable terrain.
- `walls_atlas.png`: wall runs, corners, doors, windows, roof edges, and cutaway variants.
- `props_atlas.png`: furniture, stations, barricades, debris, containers, and interactable objects.
- `characters_atlas.png`: player, customers, workers, and animation frames.
- `ui_atlas.png`: icons, cursors, prompts, and compact HUD art.

Do not mix UI with world sprites. Do not mix high-churn prototype character frames into stable building atlases.

## Metadata

Each atlas frame needs stable metadata outside the image:

- Stable asset id.
- Source file path.
- Frame rectangle.
- Anchor preset.
- Logical size or scale.
- Universe layer.
- Default depth offset.
- Optional footprint.
- Optional cutaway rule.
- Optional animation tag and frame duration.

The atlas image stores pixels only. Runtime behavior comes from metadata.

## Packing Rules

- Start with 2048 x 2048 atlases.
- Move to 4096 x 4096 only when a group exceeds the first size.
- Use at least 2 px padding between frames.
- Keep source PNGs in folders by domain even after packing.
- Keep frame ids stable when repacking so saved data and RON files do not break.

## Bevy Import Direction

Short term, continue loading individual images through `AssetServer`.

Next step, introduce an atlas registry resource that maps stable ids to Bevy `TextureAtlasLayout` indices. World data should then reference stable ids rather than raw file paths.

Long term, generate the atlas image and metadata from source folders as a build or content-pipeline step. The generated files should be treated as rebuildable artifacts, while source PNGs and metadata remain the authored content.
