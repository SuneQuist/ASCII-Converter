use image24::io::Reader as ImageReader;
use image24::imageops::FilterType;
use image_ascii::TextGenerator;
use std::{path::Path};
use std::fs::File;
use std::io::prelude::*;
use std::borrow::Cow;
use tauri::{AppHandle, Emitter};

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

    let resized = image.resize(
        if width > 0 { width} else { image.width() }, 
        if height > 0 { height } else { image.height() }, 
        FilterType::Lanczos3);

    let result: String = TextGenerator::new(&resized)
        .set_include_alpha(false)
        .set_density_chars(&[' ', '/', '%', '#'])
        .ok_or_else(|| "Invalid density character list".to_string())?
        .generate();

    let script = format!(
        "#!/bin/bash\ncat <<'ASCII_ART'\n{}\nASCII_ART\n",
        result
    );

    let mut file = File::create(output)
        .map_err(|e| e.to_string())?;

    file.write_all(script.as_bytes())
        .map_err(|e| e.to_string())?;

    app.emit("process-done", "done")
        .map_err(|e| e.to_string())?;

    Ok(())
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
