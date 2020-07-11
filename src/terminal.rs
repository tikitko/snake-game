use std::io::{stdout, Write, Stdout};
use crossterm::{ExecutableCommand, terminal, cursor, style, Result};
use std::fmt::Display;
use crossterm::style::{StyledContent, ContentStyle};

pub trait TerminalPixel {
    type Symbol: Display + Copy;
    fn symbol(&self) -> Self::Symbol;
}

type TerminalMatrix<P> where P: TerminalPixel = Vec<Vec<P>>;

pub struct Terminal {
    stdout: Stdout
}

impl Terminal {
    pub fn new() -> Self {
        Terminal { stdout: stdout() }
    }
    pub fn render(&mut self, matrix: &TerminalMatrix<impl TerminalPixel>) -> Result<()> {
        self.stdout.execute(terminal::Clear(
            terminal::ClearType::All
        ))?;
        for (i, row) in matrix.iter().enumerate() {
            for (j, element) in row.iter().enumerate() {
                self.stdout
                    .execute(cursor::MoveTo(
                        i as u16,
                        j as u16
                    ))?
                    .execute(style::PrintStyledContent(
                        StyledContent::new(ContentStyle::new(), element.symbol())
                    ))?;
            }
        }
        self.stdout.execute(cursor::MoveTo(
            0,
            0
        ))?;
        self.stdout.flush()?;
        Ok(())
    }
}

/*fn start_controls_listener(
    stop_key_code: KeyCode,
    key_code_arc: &Arc<Mutex<KeyCode>>
) -> JoinHandle<Result<()>> {
    let key_code_arc = Arc::clone(&key_code_arc);
    thread::spawn(move || {
        match enable_raw_mode() {
            Ok(_) => {},
            Err(err) => return Err(err)
        }
        loop {
            match read() {
                Ok(event) => match event {
                    Event::Key(event) => {
                        if let Ok(mut key_code_mutex) = key_code_arc.lock() {
                            let event_key_code = event.code;
                            *key_code_mutex = event_key_code;
                        }
                    },
                    _ => {}
                },
                Err(err) => return Err(err)
            }
        }
        match disable_raw_mode() {
            Ok(_) => {},
            Err(err) => return Err(err)
        }
        Ok(())
    })
}*/