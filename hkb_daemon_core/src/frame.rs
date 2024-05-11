pub type FrameSequence = Vec<Frame>;

const FRAME_DATA_SIZE: u16 = 16380;

#[repr(C)]
pub struct Frame {
    pub size: u16,                            // the size of the data
    pub frame_number: u8, // max 255 packets, so a total of 4 MB of related data (256 * 16)
    pub related_frames: u8, // the amount of related frames that we have
    pub data: [u8; FRAME_DATA_SIZE as usize], // we allow 16kb info per frame (we deduct 2 + 1 + 1 meta info)
}

impl Frame {
    pub fn from_string(data: impl AsRef<str>) -> FrameSequence {
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
                size: data_len as u16,
                frame_number: i as u8,
                related_frames: capacity as u8,
                data: frame_data,
            };

            sequence.push(frame);
        }

        sequence
    }

    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.data[0..(self.size as usize)]).into_owned()
    }
}

pub fn create_frame_from_bytes(buffer: [u8; 16384]) -> Frame {
    unsafe { std::mem::transmute::<[u8; 16384], Frame>(buffer) }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn it_can_create_a_frame_from_bytes() {
        let mut buffer: [u8; 16384] = [0; 16384];
        let str = "Hello there my friends! Hope we see you soon";
        let data = str.as_bytes();

        buffer[0..=1].copy_from_slice(&((data.len() as u16).to_le_bytes()));
        buffer[2] = 1;
        buffer[3] = 2;
        buffer[4..data.len() + 4].copy_from_slice(data);

        let frame = create_frame_from_bytes(buffer);

        assert_eq!(str.len() as u16, frame.size);
        assert_eq!(1, frame.frame_number);
        assert_eq!(2, frame.related_frames);
        assert_eq!(str, &frame.to_string())
    }

    #[test]
    fn it_can_create_a_frame_from_a_string() {
        let str = "Hello there my friends! Hope we see you soon";

        let frames = Frame::from_string(str);

        assert_eq!(1, frames.len());

        let frame = &frames[0];

        assert_eq!(str.len() as u16, frame.size);
        assert_eq!(1, frame.frame_number);
        assert_eq!(1, frame.related_frames);
        assert_eq!(str, frame.to_string());
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

            assert_eq!(part.len() as u16, frame.size);
            assert_eq!((i + 1) as u8, frame.frame_number);
            assert_eq!(expected_total_frames, frame.related_frames);
            assert_eq!(part, frame.to_string());
        }

        // assert that we have reached the end of the string and we didn't skip anyhthing
        assert_eq!(str.len(), end);
    }
}
