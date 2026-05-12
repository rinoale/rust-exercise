use std::collections::HashSet;
use std::fs;
use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use serde::Deserialize;

const IMPORTS_PATH: &str = "data/actors/turnaround_imports.ron";
const REQUIRED_COLUMNS: usize = 8;
const VALID_SUFFIXES: [&str; 8] = ["n", "ne", "e", "se", "s", "sw", "w", "nw"];

fn main() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let import_file = read_import_file(root)?;

    for import in import_file.imports {
        import_turnaround(root, &import)?;
    }

    Ok(())
}

#[derive(Deserialize)]
struct TurnaroundImportsFile {
    imports: Vec<TurnaroundImportFile>,
}

#[derive(Deserialize)]
struct TurnaroundImportFile {
    name: String,
    source: String,
    output_dir: String,
    crop_top: u32,
    crop_bottom: u32,
    columns: Vec<TurnaroundColumnFile>,
}

#[derive(Clone, Deserialize)]
struct TurnaroundColumnFile {
    index: usize,
    degree: u16,
    suffix: String,
}

#[derive(Clone, Copy)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn read_import_file(root: &Path) -> Result<TurnaroundImportsFile, String> {
    let path = root.join(IMPORTS_PATH);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

    ron::de::from_str(&source)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn import_turnaround(root: &Path, import: &TurnaroundImportFile) -> Result<(), String> {
    validate_import(import)?;

    let source_path = root.join(&import.source);
    let image = image::open(&source_path)
        .map_err(|error| format!("failed to load {}: {error}", source_path.display()))?;
    validate_crop(&image, import)?;

    for column in &import.columns {
        let rect = column_rect(&image, import, column.index);
        let output = extract_cell(&image, rect, output_cell_width(&image));
        validate_alpha(&import.name, column, &output)?;
        let output_path = root
            .join(&import.output_dir)
            .join(format!("idle_{}.png", column.suffix));
        save_image(output, &output_path)?;

        println!(
            "{}: column {} / {} deg -> {}",
            import.name,
            column.index,
            column.degree,
            output_path
                .strip_prefix(root)
                .unwrap_or(&output_path)
                .display()
        );
    }

    Ok(())
}

fn validate_import(import: &TurnaroundImportFile) -> Result<(), String> {
    if import.columns.len() != REQUIRED_COLUMNS {
        return Err(format!(
            "{} must define exactly {REQUIRED_COLUMNS} columns, got {}",
            import.name,
            import.columns.len()
        ));
    }

    let mut indexes = HashSet::new();
    let mut suffixes = HashSet::new();

    for column in &import.columns {
        if column.index >= REQUIRED_COLUMNS {
            return Err(format!(
                "{} has invalid column index {}; expected 0..{}",
                import.name,
                column.index,
                REQUIRED_COLUMNS - 1
            ));
        }

        if !indexes.insert(column.index) {
            return Err(format!(
                "{} has duplicate source column {}",
                import.name, column.index
            ));
        }

        if !VALID_SUFFIXES.contains(&column.suffix.as_str()) {
            return Err(format!(
                "{} has invalid direction suffix '{}'",
                import.name, column.suffix
            ));
        }

        if !suffixes.insert(column.suffix.as_str()) {
            return Err(format!(
                "{} has duplicate direction suffix '{}'",
                import.name, column.suffix
            ));
        }
    }

    for suffix in VALID_SUFFIXES {
        if !suffixes.contains(suffix) {
            return Err(format!(
                "{} is missing direction suffix '{}'",
                import.name, suffix
            ));
        }
    }

    Ok(())
}

fn validate_crop(image: &DynamicImage, import: &TurnaroundImportFile) -> Result<(), String> {
    let (_, height) = image.dimensions();
    if import.crop_top >= import.crop_bottom || import.crop_bottom > height {
        return Err(format!(
            "{} has invalid crop y-range {}..{} for source height {}",
            import.name, import.crop_top, import.crop_bottom, height
        ));
    }

    Ok(())
}

fn column_rect(image: &DynamicImage, import: &TurnaroundImportFile, index: usize) -> Rect {
    let (width, _) = image.dimensions();
    let x0 = width * index as u32 / REQUIRED_COLUMNS as u32;
    let x1 = width * (index as u32 + 1) / REQUIRED_COLUMNS as u32;

    Rect {
        x: x0,
        y: import.crop_top,
        width: x1 - x0,
        height: import.crop_bottom - import.crop_top,
    }
}

fn output_cell_width(image: &DynamicImage) -> u32 {
    let (width, _) = image.dimensions();
    (width + REQUIRED_COLUMNS as u32 - 1) / REQUIRED_COLUMNS as u32
}

fn extract_cell(
    image: &DynamicImage,
    rect: Rect,
    output_width: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut output = ImageBuffer::new(output_width, rect.height);
    let output_x = (output_width - rect.width) / 2;

    for y in 0..rect.height {
        for x in 0..rect.width {
            let source = image.get_pixel(rect.x + x, rect.y + y);
            let [red, green, blue, _] = source.0;
            let pixel = if is_chroma_background(red, green, blue) {
                Rgba([0, 0, 0, 0])
            } else {
                Rgba([red, green, blue, 255])
            };
            output.put_pixel(output_x + x, y, pixel);
        }
    }

    output
}

fn validate_alpha(
    import_name: &str,
    column: &TurnaroundColumnFile,
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
            "{import_name} column {} / {} deg did not extract a usable alpha sprite: {opaque} opaque, {transparent} transparent",
            column.index, column.degree
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
        let import = TurnaroundImportFile {
            name: "test".to_string(),
            source: "source.png".to_string(),
            output_dir: "out".to_string(),
            crop_top: 0,
            crop_bottom: 10,
            columns: vec![
                column(0, 0, "ne"),
                column(1, 45, "e"),
                column(2, 90, "se"),
                column(3, 135, "s"),
                column(4, 180, "sw"),
                column(5, 225, "w"),
                column(6, 270, "nw"),
                column(7, 315, "n"),
            ],
        };

        validate_import(&import).expect("complete 8-direction mapping should be valid");
    }

    #[test]
    fn import_validation_rejects_duplicate_direction_suffixes() {
        let import = TurnaroundImportFile {
            name: "test".to_string(),
            source: "source.png".to_string(),
            output_dir: "out".to_string(),
            crop_top: 0,
            crop_bottom: 10,
            columns: vec![
                column(0, 0, "ne"),
                column(1, 45, "e"),
                column(2, 90, "se"),
                column(3, 135, "s"),
                column(4, 180, "sw"),
                column(5, 225, "w"),
                column(6, 270, "w"),
                column(7, 315, "n"),
            ],
        };

        assert!(validate_import(&import).is_err());
    }

    fn column(index: usize, degree: u16, suffix: &str) -> TurnaroundColumnFile {
        TurnaroundColumnFile {
            index,
            degree,
            suffix: suffix.to_string(),
        }
    }
}
