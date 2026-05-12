use std::collections::HashSet;
use std::fs;
use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, imageops};
use serde::Deserialize;

const IMPORTS_PATH: &str = "data/actors/walk_cycle_imports.ron";
const REQUIRED_FRAMES: usize = 6;
const VALID_SUFFIXES: [&str; 8] = ["n", "ne", "e", "se", "s", "sw", "w", "nw"];

fn main() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let import_file = read_import_file(root)?;
    validate_imports(&import_file.imports)?;

    for import in import_file.imports {
        import_walk_cycle(root, &import)?;
    }

    Ok(())
}

#[derive(Deserialize)]
struct WalkCycleImportsFile {
    imports: Vec<WalkCycleImportFile>,
}

#[derive(Deserialize)]
struct WalkCycleImportFile {
    name: String,
    source: String,
    output_dir: String,
    crop_top: u32,
    crop_bottom: u32,
    suffix: String,
    output_size: Vec<u32>,
}

#[derive(Clone, Copy)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn read_import_file(root: &Path) -> Result<WalkCycleImportsFile, String> {
    let path = root.join(IMPORTS_PATH);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

    ron::de::from_str(&source)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn import_walk_cycle(root: &Path, import: &WalkCycleImportFile) -> Result<(), String> {
    let output_size = output_size(import)?;
    let source_path = root.join(&import.source);
    let image = image::open(&source_path)
        .map_err(|error| format!("failed to load {}: {error}", source_path.display()))?;
    validate_crop(&image, import)?;

    for frame_index in 0..REQUIRED_FRAMES {
        let rect = frame_rect(&image, import, frame_index);
        let cell = extract_cell(&image, rect);
        validate_alpha(&import.name, frame_index, &cell)?;
        let frame = normalize_frame(&cell, output_size).ok_or_else(|| {
            format!(
                "{} frame {frame_index} has no visible character pixels",
                import.name
            )
        })?;
        let output_path = root
            .join(&import.output_dir)
            .join(format!("walk_{}_{frame_index:02}.png", import.suffix));
        save_image(frame, &output_path)?;

        println!(
            "{}: frame {:02} -> {}",
            import.name,
            frame_index,
            output_path
                .strip_prefix(root)
                .unwrap_or(&output_path)
                .display()
        );
    }

    Ok(())
}

fn validate_imports(imports: &[WalkCycleImportFile]) -> Result<(), String> {
    let mut suffixes = HashSet::new();

    for import in imports {
        if !VALID_SUFFIXES.contains(&import.suffix.as_str()) {
            return Err(format!(
                "{} has invalid direction suffix '{}'",
                import.name, import.suffix
            ));
        }

        if !suffixes.insert(import.suffix.as_str()) {
            return Err(format!(
                "duplicate walk-cycle direction suffix '{}'",
                import.suffix
            ));
        }

        output_size(import)?;
    }

    for suffix in VALID_SUFFIXES {
        if !suffixes.contains(suffix) {
            return Err(format!("missing walk-cycle direction suffix '{suffix}'"));
        }
    }

    Ok(())
}

fn output_size(import: &WalkCycleImportFile) -> Result<(u32, u32), String> {
    if import.output_size.len() != 2 {
        return Err(format!(
            "{} output_size must have exactly 2 values, got {:?}",
            import.name, import.output_size
        ));
    }

    let width = import.output_size[0];
    let height = import.output_size[1];
    if width == 0 || height == 0 {
        return Err(format!(
            "{} output_size values must be positive, got {:?}",
            import.name, import.output_size
        ));
    }

    Ok((width, height))
}

fn validate_crop(image: &DynamicImage, import: &WalkCycleImportFile) -> Result<(), String> {
    let (_, height) = image.dimensions();
    if import.crop_top >= import.crop_bottom || import.crop_bottom > height {
        return Err(format!(
            "{} has invalid crop y-range {}..{} for source height {}",
            import.name, import.crop_top, import.crop_bottom, height
        ));
    }

    Ok(())
}

fn frame_rect(image: &DynamicImage, import: &WalkCycleImportFile, index: usize) -> Rect {
    let (width, _) = image.dimensions();
    let x0 = width * index as u32 / REQUIRED_FRAMES as u32;
    let x1 = width * (index as u32 + 1) / REQUIRED_FRAMES as u32;

    Rect {
        x: x0,
        y: import.crop_top,
        width: x1 - x0,
        height: import.crop_bottom - import.crop_top,
    }
}

fn extract_cell(image: &DynamicImage, rect: Rect) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut output = ImageBuffer::new(rect.width, rect.height);

    for y in 0..rect.height {
        for x in 0..rect.width {
            let source = image.get_pixel(rect.x + x, rect.y + y);
            let [red, green, blue, _] = source.0;
            let pixel = if is_chroma_background(red, green, blue) {
                Rgba([0, 0, 0, 0])
            } else {
                Rgba([red, green, blue, 255])
            };
            output.put_pixel(x, y, pixel);
        }
    }

    output
}

fn normalize_frame(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    output_size: (u32, u32),
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let bbox = visible_bounds(image)?;
    let cropped = imageops::crop_imm(image, bbox.x, bbox.y, bbox.width, bbox.height).to_image();
    let (output_width, output_height) = output_size;
    let scale = (output_width as f32 / cropped.width() as f32)
        .min(output_height as f32 / cropped.height() as f32);
    let resized_width = ((cropped.width() as f32 * scale).round() as u32).max(1);
    let resized_height = ((cropped.height() as f32 * scale).round() as u32).max(1);
    let resized = imageops::resize(
        &cropped,
        resized_width,
        resized_height,
        imageops::FilterType::Lanczos3,
    );
    let mut output = ImageBuffer::from_pixel(output_width, output_height, Rgba([0, 0, 0, 0]));
    let output_x = (output_width - resized_width) / 2;
    let output_y = output_height - resized_height;

    for y in 0..resized_height {
        for x in 0..resized_width {
            output.put_pixel(output_x + x, output_y + y, *resized.get_pixel(x, y));
        }
    }

    Some(output)
}

fn visible_bounds(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Option<Rect> {
    let mut min_x = image.width();
    let mut min_y = image.height();
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;

    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel.0[3] == 0 {
            continue;
        }

        found = true;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    found.then_some(Rect {
        x: min_x,
        y: min_y,
        width: max_x - min_x + 1,
        height: max_y - min_y + 1,
    })
}

fn validate_alpha(
    import_name: &str,
    frame: usize,
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> Result<(), String> {
    let mut transparent = 0usize;
    let mut opaque = 0usize;

    for pixel in image.pixels() {
        if pixel.0[3] == 0 {
            transparent += 1;
        } else {
            opaque += 1;
        }
    }

    if transparent == 0 || opaque == 0 {
        return Err(format!(
            "{import_name} frame {frame} did not extract a usable alpha sprite: {opaque} opaque, {transparent} transparent"
        ));
    }

    Ok(())
}

fn is_chroma_background(red: u8, green: u8, blue: u8) -> bool {
    green >= 140 && green.saturating_sub(red) >= 70 && green.saturating_sub(blue) >= 70
}

fn save_image(image: ImageBuffer<Rgba<u8>, Vec<u8>>, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    image
        .save(path)
        .map_err(|error| format!("failed to save {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_validation_requires_all_direction_suffixes_once() {
        let imports = VALID_SUFFIXES
            .iter()
            .map(|suffix| import(suffix))
            .collect::<Vec<_>>();

        validate_imports(&imports).expect("complete walk-cycle import set should be valid");
    }

    #[test]
    fn import_validation_rejects_duplicate_direction_suffixes() {
        let mut imports = VALID_SUFFIXES
            .iter()
            .map(|suffix| import(suffix))
            .collect::<Vec<_>>();
        imports[0].suffix = imports[1].suffix.clone();

        assert!(validate_imports(&imports).is_err());
    }

    #[test]
    fn normalize_frame_trims_and_bottom_aligns_visible_pixels() {
        let mut image = ImageBuffer::from_pixel(20, 20, Rgba([0, 0, 0, 0]));
        image.put_pixel(10, 10, Rgba([255, 0, 0, 255]));

        let normalized = normalize_frame(&image, (10, 10)).expect("visible pixel should normalize");

        assert_eq!(normalized.get_pixel(5, 9).0[3], 255);
    }

    fn import(suffix: &str) -> WalkCycleImportFile {
        WalkCycleImportFile {
            name: format!("test_{suffix}"),
            source: "source.png".to_string(),
            output_dir: "out".to_string(),
            crop_top: 0,
            crop_bottom: 10,
            suffix: suffix.to_string(),
            output_size: vec![272, 562],
        }
    }
}
