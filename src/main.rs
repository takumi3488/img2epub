use clap::Parser;

use img2epub::{img2epub, Direction};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Title of the book
    title: String,

    /// Directory of the images
    directory: String,

    /// Output file name
    output: Option<String>,

    /// Direction of the book
    /// (default: LTR)
    /// (options: LTR, RTL)
    direction: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let out = match args.output {
        Some(x) => x,
        None => format!("{}.epub", args.title),
    };

    let direction = match args.direction {
        Some(x) => match x.as_str() {
            "LTR" => Direction::LTR,
            "RTL" => Direction::RTL,
            _ => panic!("Invalid direction"),
        },
        None => Direction::LTR,
    };
    img2epub(&args.title, &args.directory, &out, direction)?;

    Ok(())
}
