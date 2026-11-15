use image24::io::Reader as ImageReader;
use image24::imageops::FilterType;
use image24::DynamicImage;
use image_ascii::TextGenerator;
use std::{path::Path};
use std::fs::File;
use std::io::prelude::*;
use std::borrow::Cow;
use tauri::{AppHandle, Emitter};
use std::fmt::Write as _;

#[cfg(target_os = "windows")]
#[tauri::command]
fn to_ascii(app: AppHandle, path: String, width: u32, height: u32) -> Result<(), String> {
    let (folder, file, extension) = get_path(&path);

    let input = format!("{}/{}.{}", folder, file, extension);
    let output = format!("{}/{}.sh", folder, file);

    let image = ImageReader::open(input)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;

    let ascii = hybrid_ascii(&image, if width > 0 { width } else { image.width() });

    let script = format!(
        "#!/usr/bin/env bash\n\
        cat <<'ASCII_ART'\n\
        {}\
        ASCII_ART\n",
        ascii
    );

    std::fs::write(&output, script)
        .map_err(|e| e.to_string())?;

    app.emit("process-done", "done")
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn colored_ascii(image: &DynamicImage, width: u32) -> String {
    // Terminal characters are usually taller than they are wide.
    let height = (
        width as f32
        * image.height() as f32
        / image.width() as f32
        * 0.5
    ).round().max(1.0) as u32;

    let resized = image
        .resize_exact(width, height, FilterType::Lanczos3)
        .to_rgb8();

    // Dark -> bright, designed for a dark terminal.
    let density = b" .:-=+*#%@";

    let mut result = String::new();

    for y in 0..height {
        for x in 0..width {
            let pixel = resized.get_pixel(x, y);
            let [r, g, b] = pixel.0;

            // Calculate perceived brightness.
            let brightness =
                0.2126 * r as f32 +
                0.7152 * g as f32 +
                0.0722 * b as f32;

            let index = (
                brightness / 255.0
                * (density.len() - 1) as f32
            ).round() as usize;

            let character = density[index] as char;

            // ANSI 24-bit foreground color.
            result.push_str(&format!(
                "\x1b[38;2;{};{};{}m{}",
                r, g, b, character
            ));
        }

        // Reset terminal color after every line.
        result.push_str("\x1b[0m\n");
    }

    result
}

fn colored_blocks(image: &DynamicImage, width: u32) -> String {
    let rows = (
        width as f32
        * image.height() as f32
        / image.width() as f32
        * 0.5
    ).round().max(1.0) as u32;

    // Two vertical pixels per terminal character.
    let img = image
        .resize_exact(width, rows * 2, FilterType::Lanczos3)
        .to_rgb8();

    let mut output = String::new();

    for y in 0..rows {
        for x in 0..width {
            let top = img.get_pixel(x, y * 2).0;
            let bottom = img.get_pixel(x, y * 2 + 1).0;

            output.push_str(&format!(
                "\x1b[38;2;{};{};{}m\
                 \x1b[48;2;{};{};{}m▀",
                top[0], top[1], top[2],
                bottom[0], bottom[1], bottom[2]
            ));
        }

        output.push_str("\x1b[0m\n");
    }

    output
}

fn hybrid_ascii(image: &DynamicImage, width: u32) -> String {
    let width = width.max(1);

    // Adjust for the shape of terminal characters.
    let height = (
        width as f32
        * image.height() as f32
        / image.width() as f32
        * 0.5
    ).round().max(1.0) as u32;

    let resized = image
        .resize_exact(width, height, FilterType::Lanczos3)
        .to_rgb8();

    let density = b" .:-=+*#%@";

    let mut output = String::new();

    for y in 0..height {
        for x in 0..width {
            let [r, g, b] = resized.get_pixel(x, y).0;

            let brightness =
                0.2126 * r as f32 +
                0.7152 * g as f32 +
                0.0722 * b as f32;

            let index = (
                brightness / 255.0
                * (density.len() - 1) as f32
            ).round() as usize;

            let character = density[index] as char;

            // Darkened image color for the background.
            let bg_r = (r as f32 * 0.55) as u8;
            let bg_g = (g as f32 * 0.55) as u8;
            let bg_b = (b as f32 * 0.55) as u8;

            // Brighter image color for the ASCII text.
            let fg_r = r.saturating_add(45);
            let fg_g = g.saturating_add(45);
            let fg_b = b.saturating_add(45);

            write!(
                output,
                "\x1b[48;2;{};{};{}m\
                 \x1b[38;2;{};{};{}m{}",
                bg_r, bg_g, bg_b,
                fg_r, fg_g, fg_b,
                character
            ).unwrap();
        }

        output.push_str("\x1b[0m\n");
    }

    output
}

// Get Input- & Output-file from path, incl. extension.
fn get_path(path: &str) -> (Cow<'_, str>, Cow<'_, str>, Cow<'_, str>) {
    let path = Path::new(path); // Create Path Struct

    // Get Path Attributes
    let file: Cow<str> = path.file_stem().unwrap().to_string_lossy();
    let extension: Cow<str> = path.extension().unwrap().to_string_lossy();
    let folder: Cow<str> = path.parent().unwrap().to_string_lossy();

    // Create paths for input and output
    //let input = format!("{}/{}.{}", folder, file, extension);
    //let output = format!("{}/{}_compressed.{}", folder, file, extension);

    return (folder, file, extension);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![to_ascii])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
