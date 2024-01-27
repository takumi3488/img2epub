mod epub;

use std::{fs::File, io::BufReader, path::Path};

use chrono::Utc;
pub use epub::epub::get_metadata;
use epub::epub::{
    create_nav_file, create_opf_file, create_part_files, initialize_directory, rm_directory,
    zip_epub,
};
use epub::images::{padding_image_file, sort_image_files};
use serde_json::from_reader;
use uuid::Uuid;

pub enum Direction {
    LTR,
    RTL,
}

pub fn img2epub(
    image_dir: &str,
    out: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create metadata
    let json_path = format!("{}/metadata.json", image_dir);
    let metadata = if Path::new(&json_path).exists() {
        let buf = BufReader::new(File::open(&json_path)?);
        from_reader(buf).expect("metadata.json is invalid")
    } else {
        panic!("metadata.json is not found");
    };

    let epub_dir = format!("./epub-{}", Uuid::new_v4().to_string());
    initialize_directory(&epub_dir)?;

    // Sort image files by name
    let sorted_files = sort_image_files(image_dir);

    // Get the maximum width and height of the images
    let sizes = sorted_files.iter().map(|x| (x.width, x.height));
    let max_width: u32 = sizes.clone().map(|s| s.0).max().unwrap();
    let max_height = sizes.clone().map(|s| s.1).max().unwrap();

    // Copy image files to the epub directory
    for file in sorted_files.iter() {
        padding_image_file(
            file.clone(),
            max_width,
            max_height,
            format!("{}/OEBPS/images/{}", epub_dir, file.file_name),
        );
    }
    padding_image_file(
        sorted_files[0].clone(),
        max_width,
        max_height,
        format!("{}/OEBPS/images/cover.jpg", epub_dir),
    );

    // Create inner files of the epub
    create_nav_file(&epub_dir, max_width, max_height)?;
    create_opf_file(
        &epub_dir,
        &format!("urn:uuid:{}", Uuid::new_v4().to_string()),
        "ja-JP",
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string().as_str(),
        &sorted_files,
        max_width,
        max_height,
        &metadata
    )?;
    create_part_files(&epub_dir, &metadata.title, &sorted_files, max_width, max_height)?;

    // Zip the directory
    zip_epub(&epub_dir, out)?;

    // Remove the directory
    rm_directory(&epub_dir)?;

    Ok(())
}
