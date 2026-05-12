use std::collections::VecDeque;
use std::fs;
use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

fn main() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    for spec in sprite_specs() {
        extract_sprite(root, spec)?;
    }

    println!("extracted {} concept sprites", sprite_specs().len());
    Ok(())
}

#[derive(Clone, Copy)]
struct SpriteSpec {
    source: &'static str,
    output: &'static str,
    rect: Rect,
    matte: Matte,
}

#[derive(Clone, Copy)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy)]
struct Matte {
    background_distance: u8,
    step_distance: u8,
    max_background_chroma: u8,
    close_gap: u32,
}

fn extract_sprite(root: &Path, spec: SpriteSpec) -> Result<(), String> {
    let source_path = root.join(spec.source);
    let image = image::open(&source_path)
        .map_err(|error| format!("failed to load {}: {error}", source_path.display()))?;
    validate_rect(&image, spec)?;

    let output_path = root.join(spec.output);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    let key = estimate_key_color(&image, spec.rect);
    let background = connected_background_mask(&image, spec.rect, key, spec.matte);
    let visible =
        solidified_visible_mask(&background, spec.rect.width, spec.rect.height, spec.matte);
    let mut output = ImageBuffer::new(spec.rect.width, spec.rect.height);

    for y in 0..spec.rect.height {
        for x in 0..spec.rect.width {
            let source = image.get_pixel(spec.rect.x + x, spec.rect.y + y);
            let [red, green, blue, _] = source.0;
            let alpha = if visible[(y * spec.rect.width + x) as usize] {
                255
            } else {
                0
            };
            output.put_pixel(x, y, Rgba([red, green, blue, alpha]));
        }
    }

    trim_and_save(output, &output_path)?;
    println!("{} <- {}", spec.output, spec.source);

    Ok(())
}

fn validate_rect(image: &DynamicImage, spec: SpriteSpec) -> Result<(), String> {
    let (width, height) = image.dimensions();
    if spec.rect.x + spec.rect.width > width || spec.rect.y + spec.rect.height > height {
        return Err(format!(
            "{} crop {:?} exceeds source dimensions {}x{}",
            spec.output,
            (spec.rect.x, spec.rect.y, spec.rect.width, spec.rect.height),
            width,
            height
        ));
    }

    Ok(())
}

fn estimate_key_color(image: &DynamicImage, rect: Rect) -> [u8; 3] {
    let samples = [
        image.get_pixel(rect.x + 4, rect.y + 4),
        image.get_pixel(rect.x + rect.width - 5, rect.y + 4),
        image.get_pixel(rect.x + 4, rect.y + rect.height - 5),
        image.get_pixel(rect.x + rect.width - 5, rect.y + rect.height - 5),
    ];

    let mut red = 0u16;
    let mut green = 0u16;
    let mut blue = 0u16;

    for sample in samples {
        red += sample.0[0] as u16;
        green += sample.0[1] as u16;
        blue += sample.0[2] as u16;
    }

    [(red / 4) as u8, (green / 4) as u8, (blue / 4) as u8]
}

fn channel_distance(color: [u8; 3], key: [u8; 3]) -> u8 {
    let red = color[0].abs_diff(key[0]);
    let green = color[1].abs_diff(key[1]);
    let blue = color[2].abs_diff(key[2]);
    red.max(green).max(blue)
}

fn color_chroma(color: [u8; 3]) -> u8 {
    let min = color[0].min(color[1]).min(color[2]);
    let max = color[0].max(color[1]).max(color[2]);
    max - min
}

fn connected_background_mask(
    image: &DynamicImage,
    rect: Rect,
    key: [u8; 3],
    matte: Matte,
) -> Vec<bool> {
    let total = (rect.width * rect.height) as usize;
    let mut background = vec![false; total];
    let mut queue = VecDeque::new();

    for x in 0..rect.width {
        enqueue_background(image, rect, key, matte, x, 0, &mut background, &mut queue);
        enqueue_background(
            image,
            rect,
            key,
            matte,
            x,
            rect.height - 1,
            &mut background,
            &mut queue,
        );
    }
    for y in 0..rect.height {
        enqueue_background(image, rect, key, matte, 0, y, &mut background, &mut queue);
        enqueue_background(
            image,
            rect,
            key,
            matte,
            rect.width - 1,
            y,
            &mut background,
            &mut queue,
        );
    }

    while let Some((x, y)) = queue.pop_front() {
        let current = pixel_rgb(image, rect, x, y);

        for (next_x, next_y) in neighbors(x, y, rect.width, rect.height) {
            let index = (next_y * rect.width + next_x) as usize;
            if background[index] {
                continue;
            }

            let next = pixel_rgb(image, rect, next_x, next_y);
            if is_background_candidate(next, key, matte)
                && channel_distance(next, current) <= matte.step_distance
            {
                background[index] = true;
                queue.push_back((next_x, next_y));
            }
        }
    }

    background
}

fn enqueue_background(
    image: &DynamicImage,
    rect: Rect,
    key: [u8; 3],
    matte: Matte,
    x: u32,
    y: u32,
    background: &mut [bool],
    queue: &mut VecDeque<(u32, u32)>,
) {
    let index = (y * rect.width + x) as usize;
    if background[index] {
        return;
    }

    if is_background_candidate(pixel_rgb(image, rect, x, y), key, matte) {
        background[index] = true;
        queue.push_back((x, y));
    }
}

fn is_background_candidate(color: [u8; 3], key: [u8; 3], matte: Matte) -> bool {
    channel_distance(color, key) <= matte.background_distance
        && color_chroma(color) <= matte.max_background_chroma
}

fn pixel_rgb(image: &DynamicImage, rect: Rect, x: u32, y: u32) -> [u8; 3] {
    let pixel = image.get_pixel(rect.x + x, rect.y + y).0;
    [pixel[0], pixel[1], pixel[2]]
}

fn neighbors(x: u32, y: u32, width: u32, height: u32) -> Vec<(u32, u32)> {
    let mut result = Vec::with_capacity(4);
    if x > 0 {
        result.push((x - 1, y));
    }
    if x + 1 < width {
        result.push((x + 1, y));
    }
    if y > 0 {
        result.push((x, y - 1));
    }
    if y + 1 < height {
        result.push((x, y + 1));
    }
    result
}

fn solidified_visible_mask(
    background: &[bool],
    width: u32,
    height: u32,
    matte: Matte,
) -> Vec<bool> {
    let mut visible = background
        .iter()
        .map(|is_background| !is_background)
        .collect::<Vec<_>>();

    for _ in 0..2 {
        close_horizontal_gaps(&mut visible, width, height, matte.close_gap);
        close_vertical_gaps(&mut visible, width, height, matte.close_gap);
    }

    visible
}

fn close_horizontal_gaps(mask: &mut [bool], width: u32, height: u32, max_gap: u32) {
    if max_gap == 0 {
        return;
    }

    for y in 0..height {
        let mut x = 0;
        while x < width {
            while x < width && !mask[(y * width + x) as usize] {
                x += 1;
            }
            if x >= width {
                break;
            }
            x += 1;

            let gap_start = x;
            while x < width && !mask[(y * width + x) as usize] {
                x += 1;
            }
            if x < width {
                let gap_len = x - gap_start;
                if gap_len > 0 && gap_len <= max_gap {
                    for fill_x in gap_start..x {
                        mask[(y * width + fill_x) as usize] = true;
                    }
                }
            }
        }
    }
}

fn close_vertical_gaps(mask: &mut [bool], width: u32, height: u32, max_gap: u32) {
    if max_gap == 0 {
        return;
    }

    for x in 0..width {
        let mut y = 0;
        while y < height {
            while y < height && !mask[(y * width + x) as usize] {
                y += 1;
            }
            if y >= height {
                break;
            }
            y += 1;

            let gap_start = y;
            while y < height && !mask[(y * width + x) as usize] {
                y += 1;
            }
            if y < height {
                let gap_len = y - gap_start;
                if gap_len > 0 && gap_len <= max_gap {
                    for fill_y in gap_start..y {
                        mask[(fill_y * width + x) as usize] = true;
                    }
                }
            }
        }
    }
}

fn trim_and_save(image: ImageBuffer<Rgba<u8>, Vec<u8>>, output_path: &Path) -> Result<(), String> {
    let (width, height) = image.dimensions();
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;

    for y in 0..height {
        for x in 0..width {
            if image.get_pixel(x, y).0[3] > 12 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    if min_x > max_x || min_y > max_y {
        return Err(format!(
            "{} crop produced no visible pixels",
            output_path.display()
        ));
    }

    let padding = 3;
    min_x = min_x.saturating_sub(padding);
    min_y = min_y.saturating_sub(padding);
    max_x = (max_x + padding).min(width - 1);
    max_y = (max_y + padding).min(height - 1);

    let trim_width = max_x - min_x + 1;
    let trim_height = max_y - min_y + 1;
    let trimmed = image::imageops::crop_imm(&image, min_x, min_y, trim_width, trim_height);

    trimmed
        .to_image()
        .save(output_path)
        .map_err(|error| format!("failed to save {}: {error}", output_path.display()))
}

fn sprite_specs() -> Vec<SpriteSpec> {
    let soft = Matte {
        background_distance: 62,
        step_distance: 28,
        max_background_chroma: 16,
        close_gap: 18,
    };
    let hard = Matte {
        background_distance: 54,
        step_distance: 24,
        max_background_chroma: 16,
        close_gap: 14,
    };
    let item = Matte {
        background_distance: 62,
        step_distance: 28,
        max_background_chroma: 16,
        close_gap: 1,
    };

    vec![
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/table_round.png",
            34,
            58,
            380,
            330,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/bar_counter.png",
            505,
            38,
            610,
            290,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/refrigerator.png",
            1190,
            35,
            285,
            350,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/chair.png",
            35,
            390,
            235,
            250,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/stool.png",
            300,
            405,
            200,
            205,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/shelf_jars.png",
            488,
            375,
            335,
            300,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/cabinet.png",
            860,
            385,
            295,
            290,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/crate_stack.png",
            1120,
            390,
            360,
            285,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/bedroll.png",
            55,
            770,
            285,
            180,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/stove.png",
            365,
            710,
            225,
            250,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/water_still.png",
            610,
            705,
            330,
            285,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/radio_table.png",
            900,
            725,
            310,
            265,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/lantern.png",
            1195,
            705,
            175,
            285,
            soft,
        ),
        spec(
            "data/concepts/realistic_furniture_station_concepts.png",
            "assets/props/rug.png",
            1300,
            720,
            220,
            220,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/tiles/tavern_floor_worn_a.png",
            45,
            45,
            210,
            130,
            hard,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/tiles/tavern_floor_worn_b.png",
            300,
            45,
            210,
            130,
            hard,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/tiles/tavern_floor_worn_c.png",
            555,
            45,
            210,
            130,
            hard,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/tiles/porch_planks_worn_a.png",
            810,
            45,
            210,
            130,
            hard,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/tiles/wasteland_dirt_a.png",
            45,
            215,
            210,
            140,
            hard,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/tiles/wasteland_dirt_b.png",
            300,
            215,
            210,
            140,
            hard,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/tiles/wasteland_scrub_a.png",
            555,
            215,
            210,
            140,
            hard,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/tiles/wasteland_asphalt_a.png",
            810,
            215,
            210,
            140,
            hard,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/walls/wall_plank_back.png",
            44,
            555,
            165,
            185,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/walls/wall_plank_side.png",
            286,
            505,
            150,
            230,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/walls/wall_plank_damaged.png",
            520,
            520,
            160,
            215,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/walls/wall_corrugated.png",
            760,
            520,
            145,
            220,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/walls/wall_tarp.png",
            975,
            505,
            150,
            235,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/walls/door_frame.png",
            1205,
            505,
            115,
            235,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/walls/window_boarded.png",
            1360,
            505,
            130,
            235,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/roofs/roof_shingles.png",
            45,
            785,
            220,
            155,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/roofs/roof_wood_damaged.png",
            300,
            785,
            220,
            155,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/roofs/roof_corrugated_red.png",
            555,
            775,
            220,
            170,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/roofs/roof_corrugated_gray.png",
            810,
            775,
            220,
            170,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/roofs/roof_edge.png",
            1055,
            790,
            230,
            150,
            soft,
        ),
        spec(
            "data/concepts/realistic_tiles_walls_biome_concepts.png",
            "assets/decals/rubble_scrub_border.png",
            1245,
            790,
            280,
            150,
            soft,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/chipped_cup.png",
            55,
            70,
            170,
            160,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/tin_mug.png",
            290,
            75,
            160,
            160,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/bottle.png",
            520,
            50,
            145,
            190,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/bowl_soup.png",
            735,
            75,
            155,
            150,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/chicken_egg.png",
            990,
            70,
            140,
            160,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/cracked_egg.png",
            1215,
            70,
            205,
            165,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/log.png",
            50,
            310,
            210,
            145,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/firewood_bundle.png",
            300,
            305,
            210,
            145,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/dead_grass.png",
            550,
            305,
            180,
            155,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/metal_ingot.png",
            1000,
            300,
            170,
            150,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/scrap_gears.png",
            1245,
            295,
            220,
            160,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/canned_food.png",
            740,
            540,
            165,
            155,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/oil_lamp.png",
            45,
            755,
            175,
            200,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/bandage_roll.png",
            510,
            790,
            190,
            140,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/seed_pouch.png",
            760,
            775,
            175,
            170,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/dirty_coin.png",
            1000,
            775,
            170,
            170,
            item,
        ),
        spec(
            "data/concepts/realistic_small_object_concepts.png",
            "assets/items/broken_lock.png",
            1220,
            755,
            230,
            190,
            item,
        ),
    ]
}

fn spec(
    source: &'static str,
    output: &'static str,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    matte: Matte,
) -> SpriteSpec {
    SpriteSpec {
        source,
        output,
        rect: Rect {
            x,
            y,
            width,
            height,
        },
        matte,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_removes_only_border_connected_background() {
        let background = [32, 32, 30];
        let object = [116, 78, 48];
        let mut image = ImageBuffer::from_pixel(
            5,
            5,
            Rgba([background[0], background[1], background[2], 255]),
        );

        for y in 1..4 {
            for x in 1..4 {
                image.put_pixel(x, y, Rgba([object[0], object[1], object[2], 255]));
            }
        }
        image.put_pixel(
            2,
            2,
            Rgba([background[0], background[1], background[2], 255]),
        );

        let mask = connected_background_mask(
            &DynamicImage::ImageRgba8(image),
            Rect {
                x: 0,
                y: 0,
                width: 5,
                height: 5,
            },
            background,
            Matte {
                background_distance: 4,
                step_distance: 4,
                max_background_chroma: 4,
                close_gap: 1,
            },
        );

        assert!(mask[0]);
        assert!(mask[4]);
        assert!(!mask[1 * 5 + 1]);
        assert!(!mask[2 * 5 + 2]);
    }
}
