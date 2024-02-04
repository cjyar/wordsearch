use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// File containing list of words to make into a wordsearch puzzle
    #[arg(short = 'f', long = "file", default_value = "words.txt")]
    pub wordlist: PathBuf,

    /// Output image file. Defaults to <wordlist>.png
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Width of wordsearch grid, in letters
    #[arg(short = 'c', long = "columns")]
    pub grid_width: Option<usize>,

    /// Height of wordsearch grid, in letters
    #[arg(short = 'r', long = "rows")]
    pub grid_height: Option<usize>,

    /// Width of produced image
    #[arg(short = 'x', long, default_value = "768")]
    pub image_width: u32,

    /// Height of produced image
    #[arg(short = 'y', long, default_value = "1024")]
    pub image_height: u32,
}
