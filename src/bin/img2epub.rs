use clap::Parser;

use img2epub::img2epub;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Directory of the images
    directory: String,

    /// Output file name
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let out = match args.output {
        Some(x) => x,
        None => format!("{}.epub", args.directory),
    };

    img2epub(&args.directory, &out)?;

    Ok(())
}
