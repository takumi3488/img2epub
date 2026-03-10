use super::images::Image;
use anyhow::{anyhow, Result};
use epub::doc::EpubDoc;
use serde::Deserialize;
use std::{
    env,
    fs::{create_dir, remove_dir_all, write},
    path::Path,
    process::Command,
};

#[derive(Deserialize)]
pub struct Metadata {
    pub title: String,
    pub creator: Option<String>,
    pub publisher: Option<String>,
    pub date: Option<String>,
    pub is_rtl: bool,
    pub blank: Option<bool>,
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

/// # Errors
///
/// Returns an error if the directory or its contents cannot be removed.
pub fn rm_directory(dir: &str) -> Result<()> {
    // Remove the directory if it exists
    if Path::new(dir).exists() {
        remove_dir_all(dir)?;
    }

    Ok(())
}

/// # Errors
///
/// Returns an error if any directory or file creation fails.
pub fn initialize_directory(dir: &str) -> Result<()> {
    // Create the directory and subdirectories
    create_dir(dir)?;
    create_dir(format!("{dir}/OEBPS"))?;
    create_dir(format!("{dir}/OEBPS/images"))?;
    create_dir(format!("{dir}/META-INF"))?;

    // Create the mimetype file
    write(format!("{dir}/mimetype"), "application/epub+zip")?;

    // Create the container.xml file
    write(
        format!("{dir}/META-INF/container.xml"),
        r#"<?xml version="1.0" encoding="UTF-8" ?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
    <rootfiles>
        <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
    </rootfiles>
</container>"#,
    )?;

    Ok(())
}

/// # Errors
///
/// Returns an error if writing the nav or CSS files fails.
pub fn create_nav_file(dir: &str, width: u32, height: u32) -> Result<()> {
    // Create the nav.xhtml file
    write(
        format!("{dir}/OEBPS/nav.xhtml"),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" ?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
    <head>
        <title>nav</title>
        <meta name="viewport" content="width={width}, height={height}"/>
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
</html>"#,
        ),
    )?;

    // Create the reset.css file
    write(
        format!("{dir}/OEBPS/reset.css"),
        r"html {color: #000; background: #FFF;}
body,div,dl,dt,dd,ul,ol,li,h1,h2,h3,h4,h5,h6,th,td {margin: 0; padding: 0;}
table {border-collapse: collapse; border-spacing: 0;}
fieldset,img {border: 0;}
caption,th,var {font-style: normal; font-weight: normal;}
li {list-style: none;}
caption,th {text-align: left;}
h1,h2,h3,h4,h5,h6 {font-size: 100%; font-weight: normal;}
sup {vertical-align: text-top;}
sub {vertical-align: text-bottom;}
a.app-amzn-magnify {display: block; width: 100%; height: 100%;}",
    )?;

    Ok(())
}

pub struct OpfParams<'a> {
    pub identifier: &'a str,
    pub language: &'a str,
    pub modified: &'a str,
    pub max_width: u32,
    pub max_height: u32,
}

/// # Errors
///
/// Returns an error if writing the OPF file fails.
pub fn create_opf_file(
    dir: &str,
    params: &OpfParams<'_>,
    images_files: &[Image],
    metadata: &Metadata,
) -> Result<()> {
    let OpfParams {
        identifier,
        language,
        modified,
        max_width,
        max_height,
    } = params;
    // Create the content.opf file
    let creator_tag = metadata
        .creator
        .as_ref()
        .map_or(String::new(), |x| format!(r"<dc:creator>{x}</dc:creator>"));
    let publisher_tag = metadata
        .publisher
        .as_ref()
        .map_or(String::new(), |x| format!(r"<dc:publisher>{x}</dc:publisher>"));
    let date_tag = metadata
        .date
        .as_ref()
        .map_or(String::new(), |x| format!(r"<dc:date>{x}</dc:date>"));
    let rtl_meta = if metadata.is_rtl {
        r#"<meta name="primary-writing-mode" content="horizontal-rl"/>"#
    } else {
        ""
    };
    let manifest_items = images_files
        .iter()
        .skip(1)
        .enumerate()
        .flat_map(|(i, x)| {
            let n = i + 1;
            vec![
                format!(r#"<item id="part{n}" href="part{n}.xhtml" media-type="application/xhtml+xml"/>"#),
                format!(
                    r#"<item id="image-{}" href="{}" media-type="image/webp"/>"#,
                    x.file_name,
                    x.relative_path()
                ),
            ]
        })
        .collect::<Vec<_>>()
        .join("\n        ");
    let spine_direction = if metadata.is_rtl {
        r#" page-progression-direction="rtl""#
    } else {
        ""
    };
    let spine_items = images_files
        .iter()
        .skip(1)
        .enumerate()
        .map(|(i, _)| {
            let n = i + 1;
            let spread = if i % 2 == usize::from(metadata.is_rtl) {
                "left"
            } else {
                "right"
            };
            format!(r#"<itemref idref="part{n}" properties="page-spread-{spread}"/>"#)
        })
        .collect::<Vec<_>>()
        .join("\n        ");

    write(
        format!("{dir}/OEBPS/content.opf"),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" ?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id">
    <metadata xmlns:opf="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/">
        <dc:identifier id="pub-id">{identifier}</dc:identifier>
        <dc:title>{}</dc:title>
        <dc:language>{language}</dc:language>{creator_tag}{publisher_tag}{date_tag}
        <meta property="dcterms:modified">{modified}</meta>
        <meta property="rendition:layout">pre-paginated</meta>
        <meta name="fixed-layout" content="true"/>
        <meta name="book-type" content="comic"/>
        <meta property="rendition:orientation">auto</meta>
        <meta property="rendition:spread">landscape</meta>
        <meta name="orientation-lock" content="auto"/>
        <meta name="original-resolution" content="{max_width}x{max_height}"/>{rtl_meta}
    </metadata>
    <manifest>
        <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
        <item id="part0" href="part0.xhtml" media-type="application/xhtml+xml"/>
        <item id="cover" href="images/cover.webp" properties="cover-image" media-type="image/webp"/>
        {manifest_items}
        <item href="reset.css" id="reset.css" media-type="text/css"/>
    </manifest>
    <spine{spine_direction}>
        <itemref idref="nav"/>
        <itemref idref="part0" properties="rendition:spread-none"/>
        {spine_items}
    </spine>
    <guide>
        <reference type="cover" title="Cover" href="part0.xhtml"/>
    </guide>
</package>"#,
            metadata.title,
        ),
    )?;

    Ok(())
}

/// # Errors
///
/// Returns an error if writing any part file fails.
pub fn create_part_files(
    dir: &str,
    title: &str,
    image_files: &[Image],
    max_width: u32,
    max_height: u32,
) -> Result<()> {
    // Create the part0.xhtml file
    write(
        format!("{dir}/OEBPS/part0.xhtml"),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" ?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
    <head>
        <title>{title}</title>
        <meta name="viewport" content="width={max_width}, height={max_height}"/>
        <link rel="stylesheet" type="text/css" href="reset.css"/>
    </head>
    <body style="font-size: 16px; height: 100%; text-align: center; width: 100%">
        {}
    </body>
</html>"#,
            format_args!(
                r#"<img src="images/cover.webp" alt="cover.webp" style="height: {max_height}px; left: 0; position: absolute; top: 0; width: {max_width}px"/>"#,
            )
        ),
    )?;

    // Create the partX.xhtml files
    for (i, file) in image_files.iter().skip(1).enumerate() {
        let n = i + 1;
        write(
            format!("{dir}/OEBPS/part{n}.xhtml"),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8" ?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
    <head>
        <title>{title}</title>
        <meta name="viewport" content="width={max_width}, height={max_height}"/>
        <link rel="stylesheet" type="text/css" href="reset.css"/>
    </head>
    <body>
        {}
    </body>
</html>"#,
                format_args!(
                    r#"<img src="{}" alt="{}" style="height: {max_height}px; left: 0; position: absolute; top: 0; width: {max_width}px"/>"#,
                    file.relative_path(),
                    file.file_name,
                )
            ),
        )?;
    }

    Ok(())
}

/// # Errors
///
/// Returns an error if zipping the directory or creating the output file fails.
pub fn zip_epub(dir: &str, out: &str) -> Result<()> {
    if Path::new(out).exists() {
        std::fs::remove_file(out)?;
    }

    let current_dir = env::current_dir()?;
    let epub_dir = if dir.starts_with('/') {
        dir.to_string()
    } else {
        format!(
            "{}/{}",
            current_dir
                .to_str()
                .ok_or_else(|| anyhow!("current dir is not valid UTF-8"))?,
            dir
        )
    };

    let out_path = if out.starts_with('/') {
        out.to_string()
    } else {
        format!(
            "{}/{}",
            current_dir
                .to_str()
                .ok_or_else(|| anyhow!("current dir is not valid UTF-8"))?,
            out
        )
    };

    println!("{epub_dir} -> {out_path}");

    Command::new("sh")
        .arg("-c")
        .arg(format!("cd {epub_dir} && zip -X0 {out_path} mimetype"))
        .output()?;
    Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {epub_dir} && zip -r9 {out_path} * -x mimetype -x .*"
        ))
        .output()?;

    Ok(())
}

/// # Errors
///
/// Returns an error if the EPUB file cannot be opened or parsed.
///
/// # Panics
///
/// Panics if the title metadata entry is missing from the EPUB.
pub fn get_metadata(file_path: &str) -> Result<Metadata, Box<dyn std::error::Error>> {
    let doc = EpubDoc::new(file_path)?;
    Ok(Metadata {
        title: doc
            .mdata("title")
            .ok_or_else(|| anyhow!("missing title in EPUB metadata"))?
            .value
            .clone(),
        creator: doc.mdata("creator").map(|x| x.value.clone()),
        publisher: doc.mdata("publisher").map(|x| x.value.clone()),
        date: doc.mdata("date").map(|x| x.value.clone()),
        is_rtl: doc
            .mdata("page-progression-direction")
            .is_some_and(|x| x.value == "rtl"),
        blank: doc.mdata("blank").map(|x| x.value == "true"),
    })
}
