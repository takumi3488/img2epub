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

pub fn sort_image_files(dir: &str) -> Vec<Image> {
    let target_files = glob(format!("{}/**/*", dir).as_str()).unwrap();
    let re = Regex::new(r"/(\D*|.*\D)(\d{1,6})\.(jpe?g|JPE?G|png|PNG|webp|WEBP)$").unwrap();
    let mut sorted_files = target_files
        .filter_map(|x| x.ok())
        .filter(|x| re.is_match(x.to_str().unwrap()))
        .collect::<Vec<_>>();
    sorted_files.sort_by(|a, b| {
        let a = re
            .captures(a.to_str().unwrap())
            .expect("invalid file name")
            .get(2)
            .unwrap()
            .as_str()
            .parse::<u32>()
            .unwrap();
        let b = re
            .captures(b.to_str().unwrap())
            .expect("invalid file name")
            .get(2)
            .unwrap()
            .as_str()
            .parse::<u32>()
            .unwrap();
        a.cmp(&b)
    });
    sorted_files
        .iter()
        .map(|x| {
            println!("Reading image: {:?}", x);
            (x, image::open(x).unwrap())
        })
        .map(|(x, imgres)| Image {
            path: x.clone(),
            file_name: format!(
                "{:06}",
                re.captures(x.to_str().unwrap())
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .parse::<u32>()
                    .unwrap(),
            ),
            width: imgres.width(),
            height: imgres.height(),
        })
        .collect::<Vec<_>>()
}

pub fn padding_image_file(
    image_file: Image,
    max_width: u32,
    max_height: u32,
    out_path: String,
) -> Image {
    let mut padding_image_file = image_file.clone();

    let width = padding_image_file.width;
    let height = padding_image_file.height;
    let width_diff = max_width - width;
    let height_diff = max_height - height;
    let padding_width = (width_diff / 2, width_diff - width_diff / 2);
    let padding_height = (height_diff / 2, height_diff - height_diff / 2);
    let img = image::open(padding_image_file.path.clone()).unwrap();
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
    )
    .unwrap();
    padding_image_file.width = max_width;
    padding_image_file.height = max_height;
    padding_image_file
}
