use std::io::{stdout, Write, Stdout};
use std::collections::HashMap;
use std::time::Duration;
use crossterm::{cursor, style, QueueableCommand, terminal, ExecutableCommand};
use crossterm::event::{read, Event, poll};
pub use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};

pub type TerminalSize = u16;
pub type TerminalPoint = (TerminalSize, TerminalSize);
pub type TerminalPixel = char;
pub type KeyCode = crossterm::event::KeyCode;
pub type ErrorKind = crossterm::ErrorKind;
pub type Result<S> = crossterm::Result<S>;

pub fn current_key_code(wait_for_duration: Duration) -> Result<KeyCode> {
    if poll(wait_for_duration)? {
        match read()? {
            Event::Key(key_event) => Ok(key_event.code),
            _ => Ok(KeyCode::Null),
        }
    } else {
        Ok(KeyCode::Null)
    }
}

pub struct Terminal {
    stdout: Stdout,
    cache: HashMap<TerminalPoint, TerminalPixel>,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
            cache: HashMap::new(),
        }
    }
    pub fn clear(&mut self) -> Result<()> {
        self.cache.clear();
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }
    pub fn render(&mut self, points_map: &HashMap<TerminalPoint, TerminalPixel>) -> Result<()> {
        const SPACE_CHAR: char = ' ';
        let mut previous_cache = self.cache.clone();
        self.cache.clear();
        for (point, pixel) in points_map {
            let is_space = *pixel == SPACE_CHAR;
            if !is_space {
                self.cache.insert(point.clone(), pixel.clone());
            }
            if let Some(previous_pixel) = previous_cache.get(&point) {
                let should_skip_render = previous_pixel == pixel;
                if !is_space {
                    previous_cache.remove(&point);
                }
                if should_skip_render {
                    continue;
                }
            }
            self.stdout
                .queue(cursor_move_to_command(point.clone()))?
                .queue(print_styled_content_command(pixel.clone()))?;
        }
        for (point, _) in previous_cache {
            self.stdout
                .queue(cursor_move_to_command(point.clone()))?
                .queue(print_styled_content_command(SPACE_CHAR))?;
        }
        self.stdout
            .queue(cursor_move_to_command((0, 0)))?;
        self.stdout.flush()?;
        Ok(())
    }
}

fn cursor_move_to_command(point: TerminalPoint) -> cursor::MoveTo {
    cursor::MoveTo(point.0, point.1)
}

fn print_styled_content_command(char: char) -> style::PrintStyledContent<char> {
    style::PrintStyledContent(style::StyledContent::new(style::ContentStyle::new(), char))
}