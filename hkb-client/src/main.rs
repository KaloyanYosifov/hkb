use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    terminal::{self, Clear},
    QueueableCommand,
};
use std::panic;
use std::{
    io::{stdout, Write},
    time::Duration,
};

fn main() {
    terminal::enable_raw_mode().unwrap();
    let (width, height) = terminal::size().unwrap();
    let mut messages = vec![];
    let mut buffer = String::with_capacity(width as usize);
    let mut stdout = stdout();
    let mut should_quit = false;

    // Return to cooked mode when app panics
    panic::set_hook(Box::new(|e| {
        // Reenter canonical mode
        // TODO: exit raw mode
        terminal::disable_raw_mode().unwrap();
        // Print panic info
        eprintln!("{e}");
    }));

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

                        messages.push(buffer.clone());
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

        for (index, message) in messages.iter().enumerate() {
            stdout.queue(cursor::MoveTo(0, index as u16)).unwrap();
            stdout.write(message.as_bytes()).unwrap();
        }

        stdout.queue(cursor::MoveTo(0, height - 2)).unwrap();
        stdout.write("-".repeat(width as usize).as_bytes()).unwrap();

        stdout.queue(cursor::MoveTo(0, height - 1)).unwrap();
        stdout.write(buffer.as_bytes()).unwrap();

        stdout.flush().unwrap();
        std::thread::sleep(Duration::from_millis(16)); // 60 FPS = 16 millis sleep
    }

    terminal::disable_raw_mode().unwrap();
}
