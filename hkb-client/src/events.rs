use crossterm::event::Event;

pub struct EventHandler {
    events: Vec<Event>,
}

impl EventHandler {
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
