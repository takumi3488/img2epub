use clap::Parser;

use img2epub::img2epub;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Directory of the images
    directory: String,

    /// Output file name
    output: Option<String>,

    /// Title of the book
    /// If not specified, the title is read from metadata.json
    #[clap(short, long)]
    title: Option<String>,

    /// Author of the book
    /// If not specified, the author is read from metadata.json
    #[clap(short, long)]
    creator: Option<String>,

    /// Publisher of the book
    /// If not specified, the publisher is read from metadata.json
    #[clap(short, long)]
    publisher: Option<String>,

    /// Date of the book
    /// If not specified, the date is read from metadata.json
    /// The format is ISO 8601 (e.g. 2021-07-04T12:34:56Z)
    #[clap(long)]
    date: Option<String>,

    /// Direction of the book.
    /// If not specified, the direction is read from metadata.json.
    /// The value is either "rtl" or "ltr".
    /// If the value is not valid nor specified, the direction is set to "ltr".
    /// If the value is "rtl", the direction is right-to-left.
    /// If the value is "ltr", the direction is left-to-right.
    #[clap(short, long)]
    direction: Option<String>,

    /// If set, add a blank page to the beginning of the book
    #[clap(short, long)]
    blank: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let out = match args.output {
        Some(x) => x,
        None => format!("{}.epub", args.directory),
    };

    img2epub(
        &args.directory,
        &out,
        args.title,
        args.creator,
        args.publisher,
        args.date,
        args.direction.map(|x| x == "rtl"),
        args.blank.then_some(true),
    )?;

    Ok(())
}
