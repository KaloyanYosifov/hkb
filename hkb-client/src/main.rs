use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    terminal::{self, Clear},
    ExecutableCommand, QueueableCommand,
};
use std::panic;
use std::{
    io::{stdout, Error as IOError, Write},
    time::Duration,
};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum RendererError {
    #[error("Lexer was unable to read the next line of the file!")]
    FailedToRenderToOutput(#[from] IOError),
}

type RenderResult = Result<(), RendererError>;

#[derive(Clone, Copy)]
struct Vector2D {
    pub x: u16,
    pub y: u16,
}

impl Vector2D {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

struct Rectangle {
    size: Vector2D,
    position: Vector2D,
}

trait Renderer {
    fn render_text(&mut self, position: Vector2D, text: impl AsRef<str>) -> RenderResult;
    fn render(&mut self, position: Vector2D, symbol: char) -> RenderResult;
}

impl<T: QueueableCommand + Write> Renderer for T {
    fn render_text(&mut self, position: Vector2D, text: impl AsRef<str>) -> RenderResult {
        self.queue(cursor::MoveTo(position.x, position.y))?;
        self.write(text.as_ref().as_bytes())?;

        Ok(())
    }

    fn render(&mut self, position: Vector2D, symbol: char) -> RenderResult {
        self.queue(cursor::MoveTo(position.x, position.y))?;
        self.write(&[symbol as u8])?;

        Ok(())
    }
}

impl Rectangle {
    pub fn new(position: Vector2D, size: Vector2D) -> Self {
        Self { position, size }
    }
}

impl Rectangle {
    pub fn render(&self, renderer: &mut impl Renderer) -> RenderResult {
        renderer.render_text(self.position.clone(), "-".repeat(self.size.x as usize))?;
        renderer.render_text(
            Vector2D {
                x: self.position.x,
                y: self.position.y + self.size.y,
            },
            "-".repeat(self.size.x as usize),
        )?;

        for y in 0..=self.size.y {
            renderer.render(
                Vector2D {
                    x: self.position.x,
                    y: y + self.position.y,
                },
                '|',
            )?;

            renderer.render(
                Vector2D {
                    x: self.position.x + self.size.x,
                    y: y + self.position.y,
                },
                '|',
            )?;
        }

        Ok(())
    }
}

fn main() {
    // Return to cooked mode when app panics
    panic::set_hook(Box::new(|e| {
        // Reenter canonical mode
        // TODO: exit raw mode
        terminal::disable_raw_mode().unwrap();

        stdout().execute(terminal::LeaveAlternateScreen).unwrap();
        // Print panic info
        eprintln!("{e}");
    }));

    let mut stdout = stdout();
    let mut should_quit = false;

    stdout.execute(terminal::EnterAlternateScreen).unwrap();
    terminal::enable_raw_mode().unwrap();

    let (width, height) = terminal::size().unwrap();
    let mut buffer = String::with_capacity(width as usize);

    let rectangle = Rectangle::new(Vector2D { x: 30, y: 20 }, Vector2D { x: 20, y: 5 });

    while !should_quit {
        while event::poll(Duration::ZERO).unwrap() {
            match event::read().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Char(c) => {
                        if c == 'c' && event.modifiers.contains(event::KeyModifiers::CONTROL) {
                            should_quit = true;
                        }

                        buffer.push(c);
                    }
                    KeyCode::Enter => {
                        if buffer == "quit" || buffer == "exit" {
                            should_quit = true;
                        }

                        buffer.clear();
                    }
                    KeyCode::Backspace => {
                        if buffer.len() > 0 {
                            buffer.remove(buffer.len() - 1);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        stdout.queue(Clear(terminal::ClearType::All)).unwrap();

        stdout
            .render_text(
                Vector2D {
                    x: 0,
                    y: height - 2,
                },
                "═".repeat(width as usize),
            )
            .unwrap();

        stdout
            .render_text(
                Vector2D {
                    x: 0,
                    y: height - 1,
                },
                &buffer,
            )
            .unwrap();

        rectangle.render(&mut stdout);

        stdout.flush().unwrap();
        std::thread::sleep(Duration::from_millis(16)); // 60 FPS = 16 millis sleep
    }

    terminal::disable_raw_mode().unwrap();
    stdout.execute(terminal::LeaveAlternateScreen).unwrap();
}
