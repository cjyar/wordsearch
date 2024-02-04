use std::{
    cmp::{max, min, Ordering},
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::anyhow;
use anyhow::Error;
use clap::Parser;
use config::Args;
use grid::Grid;
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing;
use rusttype::{Font, Scale};

mod config;
mod grid;

/// How much to pad the horizontal space allocated to each character in the grid.
const PADDING: f32 = 1.3;

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let words = read_wordlist(&args.wordlist)?;

    let grid = make_grid(&words, args.grid_width, args.grid_height)?;

    let image = make_image(&words, grid, args.image_width, args.image_height)?;

    let filename = args.output.unwrap_or_else(|| {
        let mut n = args.wordlist.clone();
        n.set_extension("png");
        n
    });
    image.save(filename)?;

    Ok(())
}

fn read_wordlist(filename: &PathBuf) -> Result<Vec<String>, Error> {
    let file = File::open(filename)?;
    let rdr = BufReader::new(file);
    let lines = rdr.lines().collect::<Result<Vec<_>, _>>()?;
    if lines.is_empty() {
        return Err(anyhow!("Empty word list: {:?}", filename));
    }
    Ok(lines)
}

fn make_grid(
    words: &[String],
    width: Option<usize>,
    height: Option<usize>,
) -> Result<Vec<Vec<char>>, Error> {
    let legal: String = ('A'..='Z').collect();
    let caps_words = words
        .iter()
        .map(|w| {
            w.to_uppercase()
                .chars()
                .filter(|c| legal.contains(*c))
                .collect()
        })
        .collect();
    let grid = Grid::new(caps_words, width, height);
    grid.generate()
}

fn make_image(
    wordlist: &Vec<String>,
    grid: Vec<Vec<char>>,
    width: u32,
    height: u32,
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Error> {
    let mut image = RgbImage::new(width, height);

    for x in 0..width {
        for y in 0..height {
            *image.get_pixel_mut(x, y) = image::Rgb([255, 255, 255]);
        }
    }

    let font = include_bytes!("../FreeSans.ttf") as &[u8];
    let font = Font::try_from_bytes(font).ok_or(anyhow!("Couldn't parse built-in font data"))?;

    let desired_stride = min(width / grid[0].len() as u32, height / grid.len() as u32);
    let text_height = compute_text_height(&font, desired_stride as i32)?;
    let scale = Scale {
        x: text_height,
        y: text_height,
    };

    // color of the text
    let (red, green, blue) = (0, 0, 0);

    let (text_width, text_height) = drawing::text_size(scale, &font, "M");
    let stride = max((text_width as f32 * PADDING) as i32, text_height);

    for (y, line) in grid.iter().enumerate() {
        for (x, letter) in line.iter().map(char::to_string).enumerate() {
            let (let_width, _) = drawing::text_size(scale, &font, &letter);
            drawing::draw_text_mut(
                &mut image,
                Rgb([red, green, blue]),
                x as i32 * stride + (stride - let_width) / 2,
                y as i32 * stride,
                scale,
                &font,
                &letter,
            );
        }
    }

    // Now make the key: the list of words hidden in the puzzle.
    let key_y0 = (grid.len() as i32 + 1) * stride;
    let scale = Scale {
        x: text_height as f32 * 0.8,
        y: text_height as f32 * 0.8,
    };
    let (_, y_stride) = drawing::text_size(scale, &font, "M");
    for ((x, y), word) in column_iter(width, y_stride as u32, 3, wordlist.len()).zip(wordlist) {
        drawing::draw_text_mut(
            &mut image,
            Rgb([red, green, blue]),
            x,
            y + key_y0,
            scale,
            &font,
            word,
        );
    }

    Ok(image)
}

/// We can't get font metrics, so we do a binary search to find an appropriate
/// text height.
fn compute_text_height(font: &Font, desired_stride: i32) -> Result<f32, Error> {
    let (mut min, mut max) = (1.0, 300.0);
    while max - min > 1.0 {
        let guess = (min + max) / 2.0;
        let scale = Scale { x: guess, y: guess };
        let (w, h) = drawing::text_size(scale, font, "M");
        let stride = core::cmp::max((w as f32 * PADDING) as i32, h);
        match stride.cmp(&desired_stride) {
            Ordering::Less => min = guess,
            Ordering::Greater => max = guess,
            Ordering::Equal => return Ok(guess),
        }
    }
    Err(anyhow!("unable to find a font size"))
}

/// Return an iterator of (X, Y) coordinates in the specified number of columns.
fn column_iter(
    image_width: u32,
    y_stride: u32,
    num_columns: u32,
    length: usize,
) -> impl Iterator<Item = (i32, i32)> {
    let mut result = vec![];
    let col_width = image_width / num_columns;
    for column in 0..num_columns {
        let mut num_rows = length as u32 / num_columns;
        if length as u32 % num_columns > column {
            num_rows += 1;
        }
        for row in 0..num_rows {
            result.push(((column * col_width) as i32, (row * y_stride) as i32));
        }
    }
    result.into_iter()
}

#[cfg(test)]
mod tests {
    use anyhow::Error;

    use crate::column_iter;

    #[test]
    fn test_column_iter() -> Result<(), Error> {
        let expecteds = vec![(0, 0), (33, 0), (66, 0)];
        for len in 0..=expecteds.len() {
            let observed: Vec<_> = column_iter(100, 10, 3, len).collect();
            let expected = expecteds[0..len].to_vec();
            assert_eq!(expected, observed);
        }

        let observed: Vec<_> = column_iter(100, 10, 3, 4).collect();
        let expected = vec![(0, 0), (0, 10), (33, 0), (66, 0)];
        assert_eq!(expected, observed);

        let observed: Vec<_> = column_iter(100, 10, 3, 5).collect();
        let expected = vec![(0, 0), (0, 10), (33, 0), (33, 10), (66, 0)];
        assert_eq!(expected, observed);

        let observed: Vec<_> = column_iter(100, 10, 3, 6).collect();
        let expected = vec![(0, 0), (0, 10), (33, 0), (33, 10), (66, 0), (66, 10)];
        assert_eq!(expected, observed);

        Ok(())
    }
}
