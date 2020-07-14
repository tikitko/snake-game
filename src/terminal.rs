use std::io::{stdout, Write, Stdout};
use crossterm::{ExecutableCommand, terminal, cursor, style, Result, QueueableCommand};
use std::fmt::Display;
use crossterm::style::{StyledContent, ContentStyle};

pub trait TerminalPixel {
    fn char(&self) -> char;
}

type TerminalMatrix<P> = Vec<Vec<P>>;

pub struct Terminal {
    stdout: Stdout,
    matrix_cache: TerminalMatrix<char>
}

impl Terminal {
    pub fn new() -> Self {
        Terminal {
            stdout: stdout(),
            matrix_cache: vec![]
        }
    }
    pub fn render(&mut self, matrix: &TerminalMatrix<impl TerminalPixel>) -> Result<()> {
        let cached_matrix = self.matrix_cache.clone();
        self.matrix_cache = vec![];
        for (i, row) in matrix.iter().enumerate() {
            self.matrix_cache.push(vec![]);
            for (j, element) in row.iter().enumerate() {
                let pixel_char = element.char();
                self.matrix_cache[i].push(pixel_char);
                if i < cached_matrix.len() {
                    if j < cached_matrix[i].len() && pixel_char == cached_matrix[i][j] {
                        continue
                    }
                }
                self.stdout
                    .queue(cursor::MoveTo(
                        i as u16,
                        j as u16
                    ))?
                    .queue(style::PrintStyledContent(
                        StyledContent::new(ContentStyle::new(), pixel_char)
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