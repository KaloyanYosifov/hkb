use crossterm::event::Event;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};

macro_rules! consume_key_event {
    ($pattern:pat $(if $guard:expr)?) => {{
        let events_consumed = events::consume_if(|event| match event {
            Event::Key(event) => match event.code {
                $pattern $(if $guard)? => true,
                _ => false,
            },
            _ => false,
        });

        events_consumed
    }};
}

macro_rules! has_key_event {
    ($pattern:pat $(if $guard:expr)?) => {{
        !events::consume_key_event!($pattern $(if $guard)?).is_empty()
    }};
}

pub(crate) use consume_key_event;
pub(crate) use has_key_event;

static GLOBAL_EVENT_HANDLER: Mutex<Option<EventHandler>> = parking_lot::const_mutex(None);

pub struct EventHandler {
    events: Vec<Event>,
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
        }
    }
}

impl EventHandler {
    pub fn all(&self) -> Vec<&Event> {
        self.events.iter().collect()
    }

    pub fn push(&mut self, event: Event) {
        self.events.push(event);
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
            .iter()
            .map(|&i| self.consume(i))
            .filter(|event| event.is_some())
            .map(|event| event.unwrap())
            .collect()
    }

    pub fn consume(&mut self, index: usize) -> Option<Event> {
        if self.events.get(index).is_none() {
            return None;
        }

        Some(self.events.swap_remove(index))
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

pub fn consume(index: usize) -> Option<Event> {
    EventHandler::get_global_handler().consume(index)
}

pub fn clear() {
    EventHandler::get_global_handler().clear()
}
