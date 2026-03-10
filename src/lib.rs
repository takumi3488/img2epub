mod epub;

use std::path::PathBuf;
use std::{fs::File, io::BufReader, path::Path};

use anyhow::{bail, Result};
use chrono::Utc;
pub use epub::converter::get_metadata;
use epub::converter::{
    create_nav_file, create_opf_file, create_part_files, initialize_directory, rm_directory,
    zip_epub, Metadata, OpfParams,
};
use epub::images::{padding_image_file, sort_image_files};
use serde_json::from_reader;
use uuid::Uuid;

pub enum Direction {
    LTR,
    RTL,
}

pub struct EpubOptions {
    pub image_dir: String,
    pub out: String,
    pub title: Option<String>,
    pub creator: Option<String>,
    pub publisher: Option<String>,
    pub publication_date: Option<String>,
    pub is_rtl: Option<bool>,
    pub blank: Option<bool>,
}

/// # Errors
///
/// Returns an error if:
/// - The metadata.json file exists but cannot be parsed.
/// - No image files are found.
/// - Title is not provided and there is no metadata.json.
/// - Any file I/O operation fails.
pub fn img2epub(opts: EpubOptions) -> Result<()> {
    let EpubOptions {
        image_dir,
        out,
        title,
        creator,
        publisher,
        publication_date,
        is_rtl,
        blank,
    } = opts;

    // Create metadata
    let json_path = format!("{image_dir}/metadata.json");
    let metadata: Metadata = if Path::new(&json_path).exists() {
        let buf = BufReader::new(File::open(&json_path)?);
        let mut meta: Metadata = from_reader(buf)?;
        meta.override_with(title, creator, publisher, publication_date, is_rtl);
        meta
    } else if let Some(ref t) = title {
        Metadata {
            title: t.clone(),
            creator,
            publisher,
            date: publication_date,
            is_rtl: is_rtl.unwrap_or(false),
            blank,
        }
    } else {
        bail!("title is required");
    };

    let epub_dir = format!("/tmp/epub-{}", Uuid::new_v4());
    initialize_directory(&epub_dir)?;

    // Sort image files by name
    let mut sorted_files = sort_image_files(&image_dir)?;

    // Get the maximum width and height of the images
    let sizes = sorted_files.iter().map(|x| (x.width, x.height));
    if sizes.clone().count() == 0 {
        bail!("No image files found");
    }
    let max_width: u32 = sizes.clone().map(|s| s.0).max().unwrap_or(0);
    let max_height = sizes.clone().map(|s| s.1).max().unwrap_or(0);

    // Create blank page
    if metadata.blank.is_some_and(|x| x) {
        let blank_page = format!("{epub_dir}/OEBPS/images/blank.webp");
        let mut imgbuf: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
            image::ImageBuffer::new(max_width, max_height);
        for x in 0..max_width {
            for y in 0..max_height {
                imgbuf.put_pixel(x, y, image::Rgb([255, 255, 255]));
            }
        }
        imgbuf.save(&blank_page)?;
        sorted_files.insert(
            1,
            epub::images::Image {
                path: PathBuf::from(&blank_page),
                file_name: "blank".to_string(),
                width: max_width,
                height: max_height,
            },
        );
    }

    // Copy image files to the epub directory
    for file in &sorted_files {
        padding_image_file(
            file,
            max_width,
            max_height,
            &format!("{epub_dir}/OEBPS/images/{}.webp", file.file_name),
        )?;
    }
    padding_image_file(
        &sorted_files[0],
        max_width,
        max_height,
        &format!("{epub_dir}/OEBPS/images/cover.webp"),
    )?;

    // Create inner files of the epub
    create_nav_file(&epub_dir, max_width, max_height)?;
    create_opf_file(
        &epub_dir,
        &OpfParams {
            identifier: &format!("urn:uuid:{}", Uuid::new_v4()),
            language: "ja-JP",
            modified: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string().as_str(),
            max_width,
            max_height,
        },
        &sorted_files,
        &metadata,
    )?;
    create_part_files(
        &epub_dir,
        &metadata.title,
        &sorted_files,
        max_width,
        max_height,
    )?;

    // Zip the directory
    zip_epub(&epub_dir, &out)?;

    // Remove the directory
    rm_directory(&epub_dir)?;

    Ok(())
}
