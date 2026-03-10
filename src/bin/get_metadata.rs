use clap::Parser;
use img2epub::get_metadata;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Epub file path
    epub: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let metadata = get_metadata(&args.epub)?;
    println!("title: {}", metadata.title);
    println!("creator: {}", metadata.creator.unwrap_or_default());
    println!("publisher: {}", metadata.publisher.unwrap_or_default());
    println!("date: {}", metadata.date.unwrap_or_default());

    Ok(())
}
