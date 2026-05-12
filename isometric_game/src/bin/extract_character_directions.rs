use std::collections::VecDeque;
use std::fs;
use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

fn main() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = root.join("data/characters.png");
    let image = image::open(&source_path)
        .map_err(|error| format!("failed to load {}: {error}", source_path.display()))?;

    for spec in traveler_a_specs() {
        extract_sprite(root, &image, spec)?;
    }

    println!(
        "extracted {} raw character direction candidates",
        traveler_a_specs().len()
    );
    Ok(())
}

#[derive(Clone, Copy)]
struct SpriteSpec {
    output: &'static str,
    rect: Rect,
}

#[derive(Clone, Copy)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn traveler_a_specs() -> Vec<SpriteSpec> {
    let y = 66;
    let width = 78;
    let height = 124;

    vec![
        // These are source-column candidates only. Do not give them canonical direction
        // names until a human validates the actual pose direction from the pixels.
        spec(
            "assets/characters/traveler_a_raw/source_00.png",
            28,
            y,
            width,
            height,
        ),
        spec(
            "assets/characters/traveler_a_raw/source_01.png",
            118,
            y,
            width,
            height,
        ),
        spec(
            "assets/characters/traveler_a_raw/source_02.png",
            210,
            y,
            width,
            height,
        ),
        spec(
            "assets/characters/traveler_a_raw/source_03.png",
            300,
            y,
            width,
            height,
        ),
        spec(
            "assets/characters/traveler_a_raw/source_04.png",
            394,
            y,
            width,
            height,
        ),
        spec(
            "assets/characters/traveler_a_raw/source_05.png",
            488,
            y,
            width,
            height,
        ),
        spec(
            "assets/characters/traveler_a_raw/source_06.png",
            582,
            y,
            width,
            height,
        ),
        spec(
            "assets/characters/traveler_a_raw/source_07.png",
            674,
            y,
            width,
            height,
        ),
    ]
}

fn spec(output: &'static str, x: u32, y: u32, width: u32, height: u32) -> SpriteSpec {
    SpriteSpec {
        output,
        rect: Rect {
            x,
            y,
            width,
            height,
        },
    }
}

fn extract_sprite(root: &Path, image: &DynamicImage, spec: SpriteSpec) -> Result<(), String> {
    validate_rect(image, spec)?;

    let output_path = root.join(spec.output);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    let key = estimate_key_color(image, spec.rect);
    let background = connected_background_mask(image, spec.rect, key);
    let mut output = ImageBuffer::new(spec.rect.width, spec.rect.height);

    for y in 0..spec.rect.height {
        for x in 0..spec.rect.width {
            let source = image.get_pixel(spec.rect.x + x, spec.rect.y + y);
            let [red, green, blue, _] = source.0;
            let alpha = if background[(y * spec.rect.width + x) as usize] {
                0
            } else {
                255
            };
            output.put_pixel(x, y, Rgba([red, green, blue, alpha]));
        }
    }

    trim_and_save(output, &output_path)?;
    println!("{}", spec.output);

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
        image.get_pixel(rect.x + 3, rect.y + 3),
        image.get_pixel(rect.x + rect.width - 4, rect.y + 3),
        image.get_pixel(rect.x + 3, rect.y + rect.height - 4),
        image.get_pixel(rect.x + rect.width - 4, rect.y + rect.height - 4),
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

fn connected_background_mask(image: &DynamicImage, rect: Rect, key: [u8; 3]) -> Vec<bool> {
    let total = (rect.width * rect.height) as usize;
    let mut background = vec![false; total];
    let mut queue = VecDeque::new();

    for x in 0..rect.width {
        enqueue_background(image, rect, key, x, 0, &mut background, &mut queue);
        enqueue_background(
            image,
            rect,
            key,
            x,
            rect.height - 1,
            &mut background,
            &mut queue,
        );
    }
    for y in 0..rect.height {
        enqueue_background(image, rect, key, 0, y, &mut background, &mut queue);
        enqueue_background(
            image,
            rect,
            key,
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
            if is_background_candidate(next, key) && channel_distance(next, current) <= 24 {
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
    x: u32,
    y: u32,
    background: &mut [bool],
    queue: &mut VecDeque<(u32, u32)>,
) {
    let index = (y * rect.width + x) as usize;
    if background[index] {
        return;
    }

    if is_background_candidate(pixel_rgb(image, rect, x, y), key) {
        background[index] = true;
        queue.push_back((x, y));
    }
}

fn is_background_candidate(color: [u8; 3], key: [u8; 3]) -> bool {
    channel_distance(color, key) <= 48 && color_chroma(color) <= 34
}

fn pixel_rgb(image: &DynamicImage, rect: Rect, x: u32, y: u32) -> [u8; 3] {
    let pixel = image.get_pixel(rect.x + x, rect.y + y).0;
    [pixel[0], pixel[1], pixel[2]]
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

    let padding = 2;
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
