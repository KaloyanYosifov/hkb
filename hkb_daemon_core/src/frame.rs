use std::mem::size_of;

use hkb_core::dtos::reminders::ReminderData;
use serde::{Deserialize, Serialize};

pub type FrameSequence = Vec<Frame>;

pub const FRAME_SIZE: usize = 16384;
const FRAME_METADATA_SIZE: usize = size_of::<FrameMetadata>();
const FRAME_DATA_SIZE: usize = FRAME_SIZE - FRAME_METADATA_SIZE;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Ping,
    Pong,
    ReminderCreated(ReminderData),
}

impl Into<FrameSequence> for Event {
    fn into(self) -> FrameSequence {
        Frame::from_event(self)
    }
}

pub struct FrameMetadata {
    size: u16,          // the size of the data
    frame_number: u8,   // max 255 packets, so a total of 4 MB of related data (256 * 16)
    related_frames: u8, // the amount of related frames that we have
}

pub struct Frame {
    metadata: FrameMetadata,
    data: [u8; FRAME_DATA_SIZE], // we allow 16kb info per frame (we deduct 2 + 1 + 1 meta info)
}

impl Frame {
    fn from_string(data: impl AsRef<str>) -> FrameSequence {
        let str = data.as_ref().as_bytes();
        let capacity = str.len().checked_div(FRAME_DATA_SIZE as usize).unwrap_or(0) + 1;
        let mut sequence: Vec<Frame> = Vec::with_capacity(capacity);

        for i in 1..=capacity {
            let start = (FRAME_DATA_SIZE as usize) * (i - 1);
            let end = std::cmp::min(str.len(), start + FRAME_DATA_SIZE as usize);
            let data = &str[start..end];
            let data_len = data.len();
            let mut frame_data = [0; FRAME_DATA_SIZE as usize];

            // copy the bytes to put in frame
            for i in 0..data_len {
                frame_data[i] = data[i];
            }

            let frame = Frame {
                metadata: FrameMetadata {
                    size: data_len as u16,
                    frame_number: i as u8,
                    related_frames: capacity as u8,
                },
                data: frame_data,
            };

            sequence.push(frame);
        }

        sequence
    }
}

impl Frame {
    pub fn from_event(event: Event) -> FrameSequence {
        Self::from_string(serde_json::to_string(&event).unwrap())
    }

    pub fn size(&self) -> u16 {
        self.metadata.size
    }

    pub fn frame_number(&self) -> u8 {
        self.metadata.frame_number
    }

    pub fn related_frames(&self) -> u8 {
        self.metadata.related_frames
    }

    pub fn data(&self) -> &[u8] {
        &self.data[0..self.size() as usize]
    }

    pub fn data_to_string(&self) -> String {
        String::from_utf8_lossy(self.data()).into_owned()
    }

    pub fn get_event(&self) -> Option<Event> {
        serde_json::from_slice::<Event>(self.data()).ok()
    }

    pub fn convert_to_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((self as *const Frame) as *const u8, FRAME_SIZE) }
    }
}

pub fn create_frame_from_bytes(buffer: [u8; FRAME_SIZE]) -> Frame {
    unsafe { std::mem::transmute::<[u8; FRAME_SIZE], Frame>(buffer) }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use hkb_core::dtos::reminders::fakes;

    #[test]
    fn it_can_create_a_frame_from_bytes() {
        let mut buffer: [u8; FRAME_SIZE] = [0; FRAME_SIZE];
        let str = "Hello there my friends! Hope we see you soon";
        let data = str.as_bytes();

        buffer[0..=1].copy_from_slice(&((data.len() as u16).to_le_bytes()));
        buffer[2] = 1;
        buffer[3] = 2;
        buffer[4..data.len() + 4].copy_from_slice(data);

        let frame = create_frame_from_bytes(buffer);

        assert_eq!(str.len() as u16, frame.size());
        assert_eq!(1, frame.frame_number());
        assert_eq!(2, frame.related_frames());
        assert_eq!(str, &frame.data_to_string())
    }

    #[test]
    fn it_can_create_a_frame_from_a_string() {
        let str = "Hello there my friends! Hope we see you soon";

        let frames = Frame::from_string(str);

        assert_eq!(1, frames.len());

        let frame = &frames[0];

        assert_eq!(str.len() as u16, frame.size());
        assert_eq!(1, frame.frame_number());
        assert_eq!(1, frame.related_frames());
        assert_eq!(str, frame.data_to_string());
    }

    #[test]
    fn it_can_create_a_frame_from_a_large_string() {
        let str = std::fs::read_to_string(Path::new("./tests/fixtures/big_file.txt")).unwrap();
        let mut end = 0;
        let frames = Frame::from_string(&str);
        let expected_total_frames = ((str.len() / FRAME_DATA_SIZE as usize) + 1) as u8;

        for (i, frame) in frames.iter().enumerate() {
            let start = (FRAME_DATA_SIZE as usize) * i;
            end = std::cmp::min(str.len(), start + FRAME_DATA_SIZE as usize);
            let part = &str[start..end];

            assert_eq!(part.len() as u16, frame.size());
            assert_eq!((i + 1) as u8, frame.frame_number());
            assert_eq!(expected_total_frames, frame.related_frames());
            assert_eq!(part, frame.data_to_string());
        }

        // assert that we have reached the end of the string and we didn't skip anyhthing
        assert_eq!(str.len(), end);
    }

    #[test]
    fn it_can_create_frame_sequence_from_event() {
        let event = Event::Ping;

        let frames = Frame::from_event(event.clone());

        assert_eq!(1, frames.len());

        let frame = &frames[0];

        assert_eq!(
            serde_json::to_string(&event).unwrap().len(),
            frame.size() as usize
        );
        assert_eq!(1, frame.frame_number());
        assert_eq!(1, frame.related_frames());

        let parsed_event: Event = serde_json::from_slice(frame.data()).unwrap();

        assert_eq!(event, parsed_event);
    }

    #[test]
    fn it_can_create_frame_sequence_from_complicated_event() {
        let event = Event::ReminderCreated(fakes::create_reminder());

        let frames: FrameSequence = event.clone().into();

        assert_eq!(1, frames.len());

        let frame = &frames[0];

        assert_eq!(
            serde_json::to_string(&event).unwrap().len(),
            frame.size() as usize
        );
        assert_eq!(1, frame.frame_number());
        assert_eq!(1, frame.related_frames());

        let parsed_event: Event = serde_json::from_slice(frame.data()).unwrap();

        assert_eq!(event, parsed_event);
    }
}
