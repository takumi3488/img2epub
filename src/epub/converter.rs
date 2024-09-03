use super::images::Image;
use epub::doc::EpubDoc;
use serde::Deserialize;
use std::{
    env,
    fs::{create_dir, remove_dir_all, write},
    path::Path,
    process::Command,
    result::Result,
};

#[derive(Deserialize)]
pub struct Metadata {
    pub title: String,
    pub creator: Option<String>,
    pub publisher: Option<String>,
    pub date: Option<String>,
    pub is_rtl: bool,
}

impl Metadata {
    pub fn override_with(
        &mut self,
        title: Option<String>,
        creator: Option<String>,
        publisher: Option<String>,
        date: Option<String>,
        is_rtl: Option<bool>,
    ) {
        if let Some(x) = title {
            self.title = x;
        }
        if let Some(x) = creator {
            self.creator = Some(x);
        }
        if let Some(x) = publisher {
            self.publisher = Some(x);
        }
        if let Some(x) = date {
            self.date = Some(x);
        }
        if let Some(x) = is_rtl {
            self.is_rtl = x;
        }
    }
}

pub fn rm_directory(dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Remove the directory if it exists
    if Path::new(dir).exists() {
        remove_dir_all(dir)?;
    }

    Ok(())
}

pub fn initialize_directory(dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create the directory and subdirectories
    create_dir(dir)?;
    create_dir(format!("{}/OEBPS", dir))?;
    create_dir(format!("{}/OEBPS/images", dir))?;
    create_dir(format!("{}/META-INF", dir))?;

    // Create the mimetype file
    write(format!("{}/mimetype", dir), "application/epub+zip")?;

    // Create the container.xml file
    write(
        format!("{}/META-INF/container.xml", dir),
        r#"<?xml version="1.0" encoding="UTF-8" ?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
    <rootfiles>
        <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
    </rootfiles>
</container>"#,
    )?;

    Ok(())
}

pub fn create_nav_file(
    dir: &str,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the nav.xhtml file
    write(
        format!("{}/OEBPS/nav.xhtml", dir),
        format!(
            r##"<?xml version="1.0" encoding="UTF-8" ?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
    <head>
        <title>nav</title>
        <meta name="viewport" content="width={}, height={}"/>
    </head>
    <body>
        <nav epub:type="toc" hidden="">
            <h1>Table of contents</h1>
            <ol>
                <li>
                    <a href="part0.xhtml">表紙</a>
                </li>
            </ol>
        </nav>
    </body>
</html>"##,
            width, height
        ),
    )?;

    // Create the reset.css file
    write(
        format!("{}/OEBPS/reset.css", dir),
        r#"html {color: #000; background: #FFF;}
body,div,dl,dt,dd,ul,ol,li,h1,h2,h3,h4,h5,h6,th,td {margin: 0; padding: 0;}
table {border-collapse: collapse; border-spacing: 0;}
fieldset,img {border: 0;}
caption,th,var {font-style: normal; font-weight: normal;}
li {list-style: none;}
caption,th {text-align: left;}
h1,h2,h3,h4,h5,h6 {font-size: 100%; font-weight: normal;}
sup {vertical-align: text-top;}
sub {vertical-align: text-bottom;}
a.app-amzn-magnify {display: block; width: 100%; height: 100%;}"#,
    )?;

    Ok(())
}

pub fn create_opf_file(
    dir: &str,
    identifier: &str,
    language: &str,
    modified: &str,
    images_files: &[Image],
    max_width: u32,
    max_height: u32,
    metadata: &Metadata,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the content.opf file
    write(
        format!("{}/OEBPS/content.opf", dir),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" ?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id">
    <metadata xmlns:opf="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/">
        <dc:identifier id="pub-id">{}</dc:identifier>
        <dc:title>{}</dc:title>
        <dc:language>{}</dc:language>{}{}{}
        <meta property="dcterms:modified">{}</meta>
        <meta property="rendition:layout">pre-paginated</meta>
        <meta name="fixed-layout" content="true"/>
        <meta name="book-type" content="comic"/>
        <meta property="rendition:orientation">auto</meta>
        <meta property="rendition:spread">landscape</meta>
        <meta name="orientation-lock" content="auto"/>
        <meta name="original-resolution" content="{}x{}"/>{}
    </metadata>
    <manifest>
        <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
        <item id="part0" href="part0.xhtml" media-type="application/xhtml+xml"/>
        <item id="cover" href="images/cover.webp" properties="cover-image" media-type="image/webp"/>
        {}
        <item href="reset.css" id="reset.css" media-type="text/css"/>
    </manifest>
    <spine{}>
        <itemref idref="nav"/>
        <itemref idref="part0" properties="rendition:spread-none"/>
        {}
    </spine>
    <guide>
        <reference type="cover" title="Cover" href="part0.xhtml"/>
    </guide>
</package>"#,
            identifier,
            metadata.title,
            language,
            if let Some(x) = &metadata.creator {
                format!(r#"<dc:creator>{}</dc:creator>"#, x)
            } else { "".to_string() },
            if let Some(x) = &metadata.publisher {
                format!(r#"<dc:publisher>{}</dc:publisher>"#, x)
            } else { "".to_string() },
            if let Some(x) = &metadata.date {
                format!(r#"<dc:date>{}</dc:date>"#, x)
            } else { "".to_string() },
            modified,
            max_width,
            max_height,
            if metadata.is_rtl {
                r#"<meta name="primary-writing-mode" content="horizontal-rl"/>"#
            } else { "" },
            images_files
                .iter()
                .skip(1)
                .enumerate()
                .flat_map(|(i,x)| vec![
                    format!(r#"<item id="part{}" href="part{}.xhtml" media-type="application/xhtml+xml"/>"#, i+1, i+1),
                    format!(
                        r#"<item id="image-{}" href="{}" media-type="image/webp"/>"#,
                        x.file_name,
                        x.relative_path()
                    )
                ])
                .collect::<Vec<_>>()
                .join("\n        "),
            if metadata.is_rtl {
                r#" page-progression-direction="rtl""#
            } else { "" },
            images_files
                .iter()
                .skip(1)
                .enumerate()
                .map(|(i,_)| format!(
                    r#"<itemref idref="part{}" properties="page-spread-{}"/>"#,
                    i+1,
                    if i % 2 == if metadata.is_rtl {1} else {0} { "left" } else { "right" }
                ))
                .collect::<Vec<_>>()
                .join("\n        "),
        ),
    )?;

    Ok(())
}

pub fn create_part_files(
    dir: &str,
    title: &str,
    image_files: &[Image],
    max_width: u32,
    max_height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the part0.xhtml file
    write(
        format!("{}/OEBPS/part0.xhtml", dir),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" ?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
    <head>
        <title>{}</title>
        <meta name="viewport" content="width={}, height={}"/>
        <link rel="stylesheet" type="text/css" href="reset.css"/>
    </head>
    <body style="font-size: 16px; height: 100%; text-align: center; width: 100%">
        {}
    </body>
</html>"#,
            title,
            max_width,
            max_height,
            format_args!(
                r#"<img src="images/cover.webp" alt="cover.webp" style="height: {}px; left: 0; position: absolute; top: 0; width: {}px"/>"#,
                max_height, max_width
            )
        ),
    )?;

    // Create the partX.xhtml files
    for (i, file) in image_files.iter().skip(1).enumerate() {
        write(
            format!("{}/OEBPS/part{}.xhtml", dir, i + 1),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8" ?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
    <head>
        <title>{}</title>
        <meta name="viewport" content="width={}, height={}"/>
        <link rel="stylesheet" type="text/css" href="reset.css"/>
    </head>
    <body>
        {}
    </body>
</html>"#,
                title,
                max_width,
                max_height,
                format_args!(
                    r#"<img src="{}" alt="{}" style="height: {}px; left: 0; position: absolute; top: 0; width: {}px"/>"#,
                    file.relative_path(),
                    file.file_name,
                    max_height,
                    max_width
                )
            ),
        )?;
    }

    Ok(())
}

pub fn zip_epub(dir: &str, out: &str) -> Result<(), Box<dyn std::error::Error>> {
    if Path::new(out).exists() {
        std::fs::remove_file(out)?;
    }

    let current_dir = env::current_dir()?;
    let epub_dir = if dir.starts_with("/") {
        dir.to_string()
    } else {
        format!("{}/{}", current_dir.to_str().unwrap(), dir)
    };

    let out_path = if out.starts_with("/") {
        out.to_string()
    } else {
        format!("{}/{}", current_dir.to_str().unwrap(), out)
    };

    println!("{} -> {}", epub_dir, out_path);

    Command::new("sh")
        .arg("-c")
        .arg(format!("cd {} && zip -X0 {} mimetype", epub_dir, out_path))
        .output()?;
    Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && zip -r9 {} * -x mimetype -x .*",
            epub_dir, out_path
        ))
        .output()?;

    Ok(())
}

pub fn get_metadata(file_path: &str) -> Result<Metadata, Box<dyn std::error::Error>> {
    let doc = EpubDoc::new(file_path)?;
    Ok(Metadata {
        title: doc.mdata("title").unwrap(),
        creator: doc.mdata("creator"),
        publisher: doc.mdata("publisher"),
        date: doc.mdata("date"),
        is_rtl: doc
            .mdata("page-progression-direction")
            .map(|x| x == "rtl")
            .unwrap_or(false),
    })
}
