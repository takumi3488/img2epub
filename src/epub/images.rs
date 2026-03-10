use std::{thread::sleep, time::Duration};

use anyhow::{anyhow, Result};
use glob::glob;
use image::GenericImageView;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Image {
    pub path: std::path::PathBuf,
    pub file_name: String,
    pub width: u32,
    pub height: u32,
}

impl Image {
    pub fn relative_path(&self) -> String {
        format!("images/{}.webp", self.file_name)
    }
}

fn extract_number(re: &Regex, path: &std::path::Path) -> Result<u32> {
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow!("invalid path (not valid UTF-8)"))?;
    let num = re
        .captures(path_str)
        .and_then(|c| c.get(2))
        .ok_or_else(|| anyhow!("invalid file name: {}", path.display()))?
        .as_str()
        .parse::<u32>()
        .map_err(|e| anyhow!("failed to parse number in file name: {e}"))?;
    Ok(num)
}

pub fn sort_image_files(dir: &str) -> Result<Vec<Image>> {
    let target_files = glob(&format!("{dir}/**/*"))?;
    let re = Regex::new(r"/(\D*|.*\D)(\d{1,6})\.(jpe?g|JPE?G|png|PNG|webp|WEBP)$")?;
    let mut sorted_files = target_files
        .filter_map(Result::ok)
        .filter(|x| x.to_str().is_some_and(|x| re.is_match(x)))
        .collect::<Vec<_>>();

    // Extract numbers for sorting, propagating errors
    let mut numbered: Vec<(u32, std::path::PathBuf)> = sorted_files
        .iter()
        .map(|x| extract_number(&re, x).map(|n| (n, x.clone())))
        .collect::<Result<Vec<_>>>()?;
    numbered.sort_by_key(|(n, _)| *n);
    sorted_files = numbered.into_iter().map(|(_, p)| p).collect();

    sorted_files
        .iter()
        .map(|x| match image::open(x.clone()) {
            Ok(image) => {
                let num = extract_number(&re, x)?;
                Ok(Image {
                    path: x.clone(),
                    file_name: format!("{num:06}"),
                    width: image.width(),
                    height: image.height(),
                })
            }
            Err(e) => Err(e.into()),
        })
        .collect::<Result<Vec<_>>>()
}

pub fn padding_image_file(
    image_file: &Image,
    max_width: u32,
    max_height: u32,
    out_path: &str,
) -> Result<Image> {
    let mut padded = image_file.clone();

    let width = padded.width;
    let height = padded.height;
    let width_diff = max_width - width;
    let height_diff = max_height - height;
    let padding_width = (width_diff / 2, width_diff - width_diff / 2);
    let padding_height = (height_diff / 2, height_diff - height_diff / 2);
    let img = image::open(padded.path.clone())?;
    let mut imgbuf: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        image::ImageBuffer::new(max_width, max_height);
    for x in 0..max_width {
        for y in 0..max_height {
            if (x < padding_width.0 || x >= max_width - padding_width.1)
                || (y < padding_height.0 || y >= max_height - padding_height.1)
            {
                imgbuf.put_pixel(x, y, image::Rgb([255, 255, 255]));
            } else {
                let rgb = img.get_pixel(x - padding_width.0, y - padding_height.0).0;
                imgbuf.put_pixel(x, y, image::Rgb([rgb[0], rgb[1], rgb[2]]));
            }
        }
    }
    image::save_buffer(
        out_path,
        &imgbuf,
        max_width,
        max_height,
        image::ColorType::Rgb8,
    )?;
    padded.width = max_width;
    padded.height = max_height;
    if out_path.contains("blank") {
        println!("{out_path}");
        sleep(Duration::from_secs(200));
    }
    Ok(padded)
}
