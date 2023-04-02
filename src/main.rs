use dicom_object::open_file;
use dicom_pixeldata::PixelDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;

fn main() {
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let input_folder = cwd.join("input");
    let output_folder = cwd.join("output");

    // Create the output folder if it doesn't exist
    if !output_folder.exists() {
        fs::create_dir(&output_folder).expect("Failed to create output folder");
    }

    let dcm_files = fs::read_dir(&input_folder)
        .expect("Failed to read input folder")
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                if entry.file_type().unwrap().is_file()
                    && entry.path().extension().map_or(false, |ext| ext == "dcm")
                {
                    Some(entry.path())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // Initialize the progress bar
    let progress_bar = ProgressBar::new(dcm_files.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{percent}% [{bar:40.cyan/blue}] {pos}/{len} {msg}").expect("Progress bar template not found")
            .progress_chars("#>-"),
    );

    // Process each DICOM file
    for file_path in dcm_files {
        let file_name = file_path
            .file_stem()
            .expect("Failed to get file stem")
            .to_string_lossy()
            .to_string();

        let output_path = output_folder.join(file_name + ".exr");

        match open_file(&file_path) {
            Ok(obj) => {
                match obj.decode_pixel_data() {
                    Ok(pixel_data) => {
                        let dynamic_image =
                            pixel_data.to_dynamic_image(0).expect("Failed to convert to dynamic image");
                        let rgba_image = dynamic_image.to_rgba32f();
                        rgba_image.save_with_format(&output_path, image::ImageFormat::OpenExr)
                            .expect("Failed to save image");
                        progress_bar.inc(1);
                    }
                    Err(err) => {
                        println!("Failed to decode pixel data from file {}: {}", file_path.display(), err);
                        progress_bar.abandon_with_message("Aborted due to error");
                        return;
                    }
                }
            }
            Err(err) => {
                println!("Failed to open file {}: {}", file_path.display(), err);
                progress_bar.abandon_with_message("Aborted due to error");
                return;
            }
        }
    }

    progress_bar.finish_with_message("Done");
}