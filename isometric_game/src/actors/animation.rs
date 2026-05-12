use std::collections::HashMap;
use std::fs;
use std::path::Path;

use bevy::prelude::*;
use serde::Deserialize;

use crate::universe::{GridPosition, IsoDirection};

pub const CHARACTER_ANIMATIONS_PATH: &str = "data/actors/character_animations.ron";

#[derive(Component)]
pub struct ActorVisualMotion {
    previous_grid: Vec2,
    phase: f32,
    pub facing: IsoDirection,
    pub moving: bool,
}

impl ActorVisualMotion {
    pub fn new(start: Vec2) -> Self {
        Self {
            previous_grid: start,
            phase: 0.0,
            facing: IsoDirection::SouthEast,
            moving: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize)]
pub enum ActorAnimationState {
    Idle,
    Walk,
    Service,
    Hurt,
    Angry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct ActorAnimationKey {
    state: ActorAnimationState,
    direction: IsoDirection,
}

#[derive(Clone)]
struct ActorAnimationClip {
    frame_seconds: f32,
    frames: Vec<String>,
}

struct CharacterAnimation {
    size: Vec2,
    clips: HashMap<ActorAnimationKey, ActorAnimationClip>,
}

#[derive(Resource, Default)]
pub struct ActorAnimationCatalog {
    characters: HashMap<String, CharacterAnimation>,
}

#[derive(Resource, Default)]
pub(super) struct ActorAnimationAssets {
    frames: HashMap<String, Handle<Image>>,
}

impl ActorAnimationAssets {
    fn get(&self, path: &str) -> Option<&Handle<Image>> {
        self.frames.get(path)
    }
}

impl ActorAnimationCatalog {
    pub(super) fn preload(&self, asset_server: &AssetServer) -> ActorAnimationAssets {
        let mut frames = HashMap::new();

        for path in self.frame_paths() {
            frames.insert(path.clone(), asset_server.load(path));
        }

        ActorAnimationAssets { frames }
    }

    fn frame_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();

        for character in self.characters.values() {
            for clip in character.clips.values() {
                paths.extend(clip.frames.iter().cloned());
            }
        }

        paths.sort();
        paths.dedup();
        paths
    }

    fn resolve(
        &self,
        character: &str,
        state: ActorAnimationState,
        direction: IsoDirection,
    ) -> Option<(ActorAnimationKey, Vec2, &ActorAnimationClip)> {
        let character = self.characters.get(character)?;

        let candidates = [
            ActorAnimationKey { state, direction },
            ActorAnimationKey {
                state,
                direction: IsoDirection::SouthEast,
            },
            ActorAnimationKey {
                state: ActorAnimationState::Idle,
                direction,
            },
            ActorAnimationKey {
                state: ActorAnimationState::Idle,
                direction: IsoDirection::SouthEast,
            },
        ];

        candidates.into_iter().find_map(|key| {
            character
                .clips
                .get(&key)
                .map(|clip| (key, character.size, clip))
        })
    }

    fn from_file(file: CharacterAnimationsFile) -> Result<Self, String> {
        let mut characters = HashMap::new();

        for character in file.characters {
            let name = character.name;
            let size = vec2(&character.size, "character animation size")?;
            let mut clips = HashMap::new();

            for clip in character.clips {
                if clip.frames.is_empty() {
                    return Err(format!(
                        "character '{name}' has an animation clip with no frames"
                    ));
                }

                for direction in clip.directions.directions() {
                    let key = ActorAnimationKey {
                        state: clip.state,
                        direction,
                    };
                    let animation_clip = ActorAnimationClip {
                        frame_seconds: clip.frame_seconds.max(0.01),
                        frames: clip.frames.clone(),
                    };

                    if clips.insert(key, animation_clip).is_some() {
                        return Err(format!(
                            "duplicate animation clip for character '{name}', state {:?}, direction {:?}",
                            key.state, key.direction
                        ));
                    }
                }
            }

            if characters
                .insert(name.clone(), CharacterAnimation { size, clips })
                .is_some()
            {
                return Err(format!("duplicate character animation entry '{name}'"));
            }
        }

        Ok(Self { characters })
    }
}

pub(super) fn preload_actor_animation_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<ActorAnimationCatalog>,
) {
    commands.insert_resource(catalog.preload(&asset_server));
}

#[derive(Component)]
pub struct ActorAnimation {
    character: String,
    active_key: Option<ActorAnimationKey>,
    requested_state: Option<ActorAnimationState>,
    frame_index: usize,
    timer: Timer,
}

impl ActorAnimation {
    pub fn new(character: impl Into<String>) -> Self {
        Self {
            character: character.into(),
            active_key: None,
            requested_state: None,
            frame_index: 0,
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }

    pub fn debug_summary(&self) -> String {
        let Some(key) = self.active_key else {
            return format!("{}: no clip", self.character);
        };

        format!(
            "{}: {:?} {:?} frame {}",
            self.character, key.state, key.direction, self.frame_index
        )
    }

    fn desired_state(&self, motion: &ActorVisualMotion) -> ActorAnimationState {
        self.requested_state.unwrap_or(if motion.moving {
            ActorAnimationState::Walk
        } else {
            ActorAnimationState::Idle
        })
    }
}

pub(super) fn update_actor_visual_motion(
    time: Res<Time>,
    mut actors: Query<(&GridPosition, &mut ActorVisualMotion, &mut Transform)>,
) {
    for (grid, mut motion, mut transform) in &mut actors {
        let delta = grid.value - motion.previous_grid;
        motion.moving = delta.length_squared() > 0.0001;

        if motion.moving {
            if let Some(direction) = IsoDirection::from_grid_delta(delta) {
                motion.facing = direction;
            }
            motion.phase += time.delta_secs() * 10.0;
        } else {
            motion.phase += time.delta_secs() * 2.5;
        }

        let bob = if motion.moving {
            motion.phase.sin() * 2.2
        } else {
            motion.phase.sin() * 0.45
        };
        let scale = if motion.moving {
            1.0 + motion.phase.sin().abs() * 0.015
        } else {
            1.0
        };

        transform.translation.y += bob;
        transform.scale = Vec3::splat(scale);
        motion.previous_grid = grid.value;
    }
}

pub(super) fn update_actor_animation(
    time: Res<Time>,
    animation_assets: Res<ActorAnimationAssets>,
    images: Res<Assets<Image>>,
    catalog: Res<ActorAnimationCatalog>,
    mut actors: Query<(&ActorVisualMotion, &mut ActorAnimation, &mut Sprite)>,
) {
    for (motion, mut animation, mut sprite) in &mut actors {
        let state = animation.desired_state(motion);
        let Some((key, size, clip)) = catalog.resolve(&animation.character, state, motion.facing)
        else {
            continue;
        };

        if animation.active_key != Some(key) {
            animation.active_key = Some(key);
            animation.frame_index = 0;
            animation.timer = Timer::from_seconds(clip.frame_seconds, TimerMode::Repeating);
            apply_animation_frame(
                &animation_assets,
                &images,
                &mut sprite,
                clip,
                animation.frame_index,
                size,
            );
            continue;
        }

        animation.timer.tick(time.delta());
        if animation.timer.just_finished() && clip.frames.len() > 1 {
            animation.frame_index = (animation.frame_index + 1) % clip.frames.len();
            apply_animation_frame(
                &animation_assets,
                &images,
                &mut sprite,
                clip,
                animation.frame_index,
                size,
            );
        }
    }
}

fn apply_animation_frame(
    animation_assets: &ActorAnimationAssets,
    images: &Assets<Image>,
    sprite: &mut Sprite,
    clip: &ActorAnimationClip,
    frame_index: usize,
    size: Vec2,
) {
    sprite.custom_size = Some(size);
    let frame_path = &clip.frames[frame_index];
    let Some(next_image) = animation_assets.get(frame_path).cloned() else {
        return;
    };

    if images.get(&next_image).is_some() {
        sprite.image = next_image;
    }
}

pub fn load_actor_animation_catalog(path: &str) -> Result<ActorAnimationCatalog, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let file: CharacterAnimationsFile = ron::de::from_str(&source)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    ActorAnimationCatalog::from_file(file)
}

#[derive(Deserialize)]
struct CharacterAnimationsFile {
    characters: Vec<CharacterAnimationFile>,
}

#[derive(Deserialize)]
struct CharacterAnimationFile {
    name: String,
    size: Vec<f32>,
    clips: Vec<AnimationClipFile>,
}

#[derive(Clone, Deserialize)]
struct AnimationClipFile {
    state: ActorAnimationState,
    #[serde(default)]
    directions: DirectionSetFile,
    frame_seconds: f32,
    frames: Vec<String>,
}

#[derive(Clone, Deserialize)]
enum DirectionSetFile {
    All,
    Only(Vec<IsoDirection>),
}

impl Default for DirectionSetFile {
    fn default() -> Self {
        Self::All
    }
}

impl DirectionSetFile {
    fn directions(&self) -> Vec<IsoDirection> {
        match self {
            DirectionSetFile::All => IsoDirection::ALL.to_vec(),
            DirectionSetFile::Only(directions) => directions.clone(),
        }
    }
}

fn vec2(value: &[f32], label: &str) -> Result<Vec2, String> {
    if value.len() != 2 {
        return Err(format!("{label} must have exactly 2 values, got {value:?}"));
    }

    Ok(Vec2::new(value[0], value[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_delta_resolves_to_eight_directions() {
        assert_eq!(
            IsoDirection::from_grid_delta(Vec2::new(1.0, 0.0)),
            Some(IsoDirection::South)
        );
        assert_eq!(
            IsoDirection::from_grid_delta(Vec2::new(1.0, 1.0)),
            Some(IsoDirection::SouthWest)
        );
        assert_eq!(
            IsoDirection::from_grid_delta(Vec2::new(0.0, -1.0)),
            Some(IsoDirection::East)
        );
        assert_eq!(
            IsoDirection::from_grid_delta(Vec2::new(-1.0, -1.0)),
            Some(IsoDirection::NorthEast)
        );
    }

    #[test]
    fn character_animation_file_loads() {
        let catalog = load_actor_animation_catalog(CHARACTER_ANIMATIONS_PATH)
            .expect("character animation metadata should load");

        let (key, size, clip) = catalog
            .resolve(
                "bartender",
                ActorAnimationState::Walk,
                IsoDirection::NorthWest,
            )
            .expect("bartender walk clip should resolve for every direction");

        assert_eq!(key.state, ActorAnimationState::Walk);
        assert_eq!(key.direction, IsoDirection::NorthWest);
        assert_eq!(size, Vec2::new(48.0, 100.0));
        assert!(!clip.frames.is_empty());
    }

    #[test]
    fn animation_catalog_lists_unique_frame_paths_for_preload() {
        let catalog = load_actor_animation_catalog(CHARACTER_ANIMATIONS_PATH)
            .expect("character animation metadata should load");
        let paths = catalog.frame_paths();

        assert!(paths.contains(&"characters/bartender_walk/walk_ne_00.png".to_string()));
        assert_eq!(
            paths.iter().collect::<std::collections::HashSet<_>>().len(),
            paths.len()
        );
    }

    #[test]
    fn clip_resolution_falls_back_to_idle_southeast() {
        let file = CharacterAnimationsFile {
            characters: vec![CharacterAnimationFile {
                name: "test".to_string(),
                size: vec![48.0, 100.0],
                clips: vec![AnimationClipFile {
                    state: ActorAnimationState::Idle,
                    directions: DirectionSetFile::Only(vec![IsoDirection::SouthEast]),
                    frame_seconds: 0.5,
                    frames: vec!["characters/bartender.png".to_string()],
                }],
            }],
        };
        let catalog = ActorAnimationCatalog::from_file(file).expect("test catalog should build");

        let (key, _, _) = catalog
            .resolve("test", ActorAnimationState::Walk, IsoDirection::North)
            .expect("missing walk north should fall back");

        assert_eq!(key.state, ActorAnimationState::Idle);
        assert_eq!(key.direction, IsoDirection::SouthEast);
    }
}
