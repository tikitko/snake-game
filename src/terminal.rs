use std::io::{stdout, Write, Stdout};
use crossterm::{cursor, style, Result, QueueableCommand};
use crossterm::style::{StyledContent, ContentStyle};
use std::collections::HashMap;
use crate::point::Point;

type TerminalPoints<P> = HashMap<Point<u16>, P>;
type TerminalMatrix<P> = Vec<Vec<P>>;

pub trait TerminalPixel {
    fn char(&self) -> char;
}

pub struct Terminal {
    stdout: Stdout,
    cache: HashMap<Point<u16>, char>
}

impl Terminal {
    pub const SPACE_CHAR: char = ' ';
    pub fn new() -> Self {
        Terminal {
            stdout: stdout(),
            cache: HashMap::new()
        }
    }
    pub fn render_points(&mut self, points: &TerminalPoints<impl TerminalPixel>) -> Result<()> {
        let mut previous_cache = self.cache.clone();
        self.cache = HashMap::new();
        for (point, pixel) in points {
            let char = pixel.char();
            if let Some(previous_char) = previous_cache.get(point) {
                if *previous_char == char {
                    previous_cache.remove(point);
                }
            }
            self.stdout
                .queue(cursor::MoveTo(
                    point.x(),
                    point.y()
                ))?
                .queue(style::PrintStyledContent(
                    StyledContent::new(ContentStyle::new(), char)
                ))?;
        }
        for (point, _) in previous_cache {
            self.stdout
                .queue(cursor::MoveTo(
                    point.x(),
                    point.y()
                ))?
                .queue(style::PrintStyledContent(
                    StyledContent::new(ContentStyle::new(), Self::SPACE_CHAR)
                ))?;
        }
        self.stdout.queue(cursor::MoveTo(
            0,
            0
        ))?;
        self.stdout.flush()?;
        Ok(())
    }
    pub fn render_matrix(&mut self, matrix: &TerminalMatrix<impl TerminalPixel>) -> Result<()> {
        let previous_cache = self.cache.clone();
        self.cache = HashMap::new();
        for (i, row) in matrix.iter().enumerate() {
            for (j, element) in row.iter().enumerate() {
                let point = Point::new(i as u16, j as u16);
                let char = element.char();
                if char != Self::SPACE_CHAR {
                    self.cache.insert(point.clone(), char);
                }
                if let Some(previous_char) = previous_cache.get(&point) {
                    if *previous_char == char {
                        continue
                    }
                }
                self.stdout
                    .queue(cursor::MoveTo(
                        point.x(),
                        point.y()
                    ))?
                    .queue(style::PrintStyledContent(
                        StyledContent::new(ContentStyle::new(), char)
                    ))?;
            }
        }
        self.stdout.queue(cursor::MoveTo(
            0,
            0
        ))?;
        self.stdout.flush()?;
        Ok(())
    }
}