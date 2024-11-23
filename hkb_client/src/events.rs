use crossterm::event::{Event, KeyCode};
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use std::time::Instant;

macro_rules! consume_key_events {
    ($pattern:pat $(if $guard:expr)?) => {{
        let events_consumed = events::consume_if(|event| match event {
            crossterm::event::Event::Key(event) => match event.code {
                $pattern $(if $guard)? => true,
                _ => false,
            },
            _ => false,
        });

        events_consumed
    }};
}

macro_rules! consume_key_event {
    ($($pattern:pat $(if $guard:expr)? => $expr:expr)+) => {
        // Disable unused variables, for the first match
        #[allow(unused_variables)]
        let events_consumed = events::consume_if(|event| match event {
            crossterm::event::Event::Key(event) => match event.code {
                $($pattern $(if $guard)? => true,)+
                _ => false,
            },
            _ => false,
        });

        for event in events_consumed {
            match event {
                Event::Key(key) => match key.code {
                    $($pattern $(if $guard)? => $expr,)+
                    _ => {}
                }
                _ => {}
            }
        }
    };
}

macro_rules! has_key_event {
    ($pattern:pat $(if $guard:expr)?) => {{
        !events::consume_key_events!($pattern $(if $guard)?).is_empty()
    }};
}

pub(crate) use consume_key_event;
pub(crate) use consume_key_events;
pub(crate) use has_key_event;

static GLOBAL_EVENT_HANDLER: Mutex<Option<EventHandler>> = parking_lot::const_mutex(None);

pub struct EventHandler {
    events: Vec<Event>,
    times_pressed: usize,
    key_release_delay: Instant,
    previous_key: Option<char>,
}

impl EventHandler {
    fn get_global_handler() -> MappedMutexGuard<'static, Self> {
        MutexGuard::map(GLOBAL_EVENT_HANDLER.lock(), |reader| {
            reader.get_or_insert_with(Self::new)
        })
    }

    pub fn new() -> Self {
        Self {
            // 10 is a random initial number here. We shouldn't be getting more than 10 events in one loop
            events: Vec::with_capacity(10),

            times_pressed: 0,
            previous_key: None,
            key_release_delay: Instant::now(),
        }
    }
}

impl EventHandler {
    #[allow(dead_code)]
    pub fn all(&self) -> Vec<&Event> {
        self.events.iter().collect()
    }

    pub fn push(&mut self, event: Event) {
        if let Event::Key(e) = event {
            if let KeyCode::Char(c) = e.code {
                // if 300 ms have passed, we assume we had a key release
                // This is a hacky way of figuring out when a key has been released
                if self.key_release_delay.elapsed().as_millis() >= 300 {
                    self.times_pressed += 1;
                }

                if let Some(prev) = self.previous_key {
                    if prev != c {
                        self.times_pressed = 1;
                    }
                }

                self.previous_key = Some(c);
            }
        };

        self.events.push(event);
        self.key_release_delay = Instant::now();
    }

    pub fn consume_if<T: Fn(&Event) -> bool>(&mut self, callback: T) -> Vec<Event> {
        // arbitrary selected capacity
        let mut events_to_consume: Vec<usize> = Vec::with_capacity(self.events.len() / 2);

        for i in 0..self.events.len() {
            if callback(&self.events[i]) {
                events_to_consume.push(i);
            }
        }

        events_to_consume
            .into_iter()
            .filter_map(|i| self.consume(i))
            .collect()
    }

    pub fn consume(&mut self, index: usize) -> Option<Event> {
        if index >= self.events.len() {
            return None;
        }

        Some(self.events.swap_remove(index))
    }

    pub fn reset_key_press(&mut self) {
        self.times_pressed = 0;
    }

    pub fn is_pressed_at_least(&self, c: char, times: usize) -> bool {
        if let Some(prev) = self.previous_key {
            prev == c && self.times_pressed >= times
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

pub fn push(event: Event) {
    EventHandler::get_global_handler().push(event)
}

pub fn consume_if<T: Fn(&Event) -> bool>(callback: T) -> Vec<Event> {
    EventHandler::get_global_handler().consume_if(callback)
}

#[allow(dead_code)]
pub fn consume(index: usize) -> Option<Event> {
    EventHandler::get_global_handler().consume(index)
}

pub fn reset_key_press() {
    EventHandler::get_global_handler().reset_key_press()
}

pub fn is_pressed_at_least(c: char, times: usize) -> bool {
    EventHandler::get_global_handler().is_pressed_at_least(c, times)
}

pub fn clear() {
    EventHandler::get_global_handler().clear()
}
