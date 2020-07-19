use std::io::{stdout, Write, Stdout};
use std::collections::HashMap;
use std::time::Duration;
use crossterm::{cursor, style, QueueableCommand, terminal};
use crossterm::style::{StyledContent, ContentStyle};
use crossterm::event::{read, Event, poll};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::point::Point;

pub type TerminalPoints<P> = HashMap<Point<u16>, P>;
pub type TerminalMatrix<P> = Vec<Vec<P>>;
pub type KeyCode = crossterm::event::KeyCode;
pub type ErrorKind = crossterm::ErrorKind;
pub type Result<S> = crossterm::Result<S>;

pub trait TerminalPixel {
    fn char(&self) -> char;
}

pub struct Terminal {
    stdout: Stdout,
    cache: HashMap<Point<u16>, char>,
}

impl Terminal {
    pub const SPACE_CHAR: char = ' ';
    pub fn new() -> Self {
        Terminal {
            stdout: stdout(),
            cache: HashMap::new(),
        }
    }
    pub fn enable_raw_mode() -> Result<()> {
        enable_raw_mode()
    }
    pub fn disable_raw_mode() -> Result<()> {
        disable_raw_mode()
    }
    pub fn current_key_code(wait_for_duration: Duration) -> Result<KeyCode> {
        if poll(wait_for_duration)? {
            return match read()? {
                Event::Key(key_event) => Ok(key_event.code),
                _ => Ok(KeyCode::Null)
            };
        } else {
            Ok(KeyCode::Null)
        }
    }
    pub fn clear(&mut self) -> Result<()> {
        self.stdout.queue(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }
    pub fn render_points<P>(&mut self, points: &TerminalPoints<P>) -> Result<()> where
        P: TerminalPixel {
        let mut previous_cache = self.cache.clone();
        self.cache = HashMap::new();
        for (point, pixel) in points {
            let char = pixel.char();
            self.cache.insert(point.clone(), char);
            if let Some(previous_char) = previous_cache.get(point) {
                let should_skip_render = *previous_char == char;
                previous_cache.remove(point);
                if should_skip_render {
                    continue;
                }
            }
            self.stdout
                .queue(cursor::MoveTo(
                    point.x(),
                    point.y(),
                ))?
                .queue(style::PrintStyledContent(
                    StyledContent::new(ContentStyle::new(), char)
                ))?;
        }
        for (point, _) in previous_cache {
            self.stdout
                .queue(cursor::MoveTo(
                    point.x(),
                    point.y(),
                ))?
                .queue(style::PrintStyledContent(
                    StyledContent::new(ContentStyle::new(), Self::SPACE_CHAR)
                ))?;
        }
        self.stdout.queue(cursor::MoveTo(
            0,
            0,
        ))?;
        self.stdout.flush()?;
        Ok(())
    }
    pub fn render_matrix<P>(&mut self, matrix: &TerminalMatrix<P>) -> Result<()> where
        P: TerminalPixel {
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
                        continue;
                    }
                }
                self.stdout
                    .queue(cursor::MoveTo(
                        point.x(),
                        point.y(),
                    ))?
                    .queue(style::PrintStyledContent(
                        StyledContent::new(ContentStyle::new(), char)
                    ))?;
            }
        }
        self.stdout.queue(cursor::MoveTo(
            0,
            0,
        ))?;
        self.stdout.flush()?;
        Ok(())
    }
}