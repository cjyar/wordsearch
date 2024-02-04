use std::cmp::max;
use std::ops::RangeInclusive;

use anyhow::{anyhow, Error};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_derive2::RandGen;

pub struct Grid {
    wordlist: Vec<String>,
    width: usize,
    height: usize,
    grid: Vec<Vec<Option<char>>>,
}

impl Grid {
    pub fn new(wordlist: Vec<String>, width: Option<usize>, height: Option<usize>) -> Self {
        let longest_word = wordlist.iter().map(String::len).max().unwrap();
        let avg_len =
            wordlist.iter().map(String::len).sum::<usize>() as f32 / wordlist.len() as f32;
        let num_letters = avg_len * wordlist.len() as f32;
        let default_size = f32::sqrt(num_letters * 2.0).ceil() as usize;
        let w = max(longest_word, width.unwrap_or(default_size));
        let h = max(longest_word, height.unwrap_or(default_size));

        Grid {
            wordlist,
            width: w,
            height: h,
            grid: vec![vec![None; w]; h],
        }
    }

    pub fn generate(self) -> Result<Vec<Vec<char>>, Error> {
        let mut rng = rand::thread_rng();
        let mut wordlist = self.wordlist.clone();
        wordlist.shuffle(&mut rng);
        let shuffled = Self { wordlist, ..self };
        let grid = shuffled.place_word(&mut rng)?.grid;
        let result = grid
            .into_iter()
            .map(|row| row.into_iter().map(|cell| cell.unwrap()).collect())
            .collect();
        Ok(result)
    }

    /// Recursively place the word at the front of wordlist, or return an error if a placement can't be found after
    /// retries.
    fn place_word(self, rng: &mut ThreadRng) -> Result<Self, Error> {
        let mut wordlist = self.wordlist.clone();
        match wordlist.pop() {
            None => self.fill(&mut *rng),
            Some(word) => {
                let retry_limit = self.empty_count();
                for _ in 0..retry_limit {
                    let dir: Direction = rng.gen();
                    let (xrange, yrange) = dir.ranges(word.len(), self.width, self.height);
                    let x = rng.gen_range(xrange);
                    let y = rng.gen_range(yrange);
                    match self.try_word(&word, dir, x, y) {
                        Err(_) => (),
                        Ok(grid) => {
                            return Self {
                                grid,
                                wordlist,
                                ..self
                            }
                            .place_word(rng);
                        }
                    }
                }
                Err(anyhow!(
                    "Failed to place {} after {} retries",
                    word,
                    retry_limit
                ))
            }
        }
    }

    /// Try to place the word into the grid. Return the new grid.
    fn try_word(
        &self,
        word: &str,
        dir: Direction,
        x0: usize,
        y0: usize,
    ) -> Result<Vec<Vec<Option<char>>>, Error> {
        // First check if we can insert it, to save copying the whole grid.
        let (mut x, mut y) = (x0, y0);
        for letter in word.chars() {
            match self.grid[y][x] {
                None => (),
                Some(x) if x == letter => (),
                _ => return Err(anyhow!("Doesn't fit.")),
            }
            let (dx, dy) = dir.next();
            x = (x as isize + dx) as usize;
            y = (y as isize + dy) as usize;
        }

        // It fits, so now actually place it.
        let mut grid = self.grid.clone();
        let (mut x, mut y) = (x0, y0);
        for letter in word.chars() {
            grid[y][x] = Some(letter);
            let (dx, dy) = dir.next();
            x = (x as isize + dx) as usize;
            y = (y as isize + dy) as usize;
        }

        Ok(grid)
    }

    /// Finish the grid by filling in random letters in all the blank spaces.
    fn fill(self, rng: &mut ThreadRng) -> Result<Self, Error> {
        let mut grid = self.grid.clone();
        for row in grid.iter_mut() {
            for cell in row.iter_mut() {
                if cell.is_none() {
                    let letter = rng.gen_range('A'..='Z');
                    *cell = Some(letter);
                }
            }
        }
        Ok(Self { grid, ..self })
    }

    /// Return the approximate number of empty cells remaining.
    fn empty_count(&self) -> usize {
        self.grid
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| cell.map_or_else(|| 1, |_| 0))
                    .sum::<usize>()
            })
            .sum()
    }
}

#[derive(RandGen)]
enum Direction {
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
    North,
    Northeast,
}

impl Direction {
    /// Return the next position after the current one, in (dx, dy) form.
    fn next(&self) -> (isize, isize) {
        match self {
            Self::East => (1, 0),
            Self::Southeast => (1, 1),
            Self::South => (0, 1),
            Self::Southwest => (-1, 1),
            Self::West => (-1, 0),
            Self::Northwest => (-1, -1),
            Self::North => (0, -1),
            Self::Northeast => (1, -1),
        }
    }

    /// Return the allowable starting positions for a word of length len.
    fn ranges(
        &self,
        len: usize,
        width: usize,
        height: usize,
    ) -> (RangeInclusive<usize>, RangeInclusive<usize>) {
        let (dx, dy) = self.next();
        let (xmin, xmax) = if dx < 0 {
            (len - 1, width - 1)
        } else {
            (0, width - len)
        };
        let (ymin, ymax) = if dy < 0 {
            (len - 1, height - 1)
        } else {
            (0, height - len)
        };
        (
            RangeInclusive::new(xmin, xmax),
            RangeInclusive::new(ymin, ymax),
        )
    }
}
