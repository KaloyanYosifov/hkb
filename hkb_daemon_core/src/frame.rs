#[repr(C)]
pub struct Frame {
    pub size: u16,          // the size of the data
    pub frame_number: u8,   // max 255 packets, so a total of 4 MB of related data (256 * 16)
    pub related_frames: u8, // the amount of related frames that we have
    pub data: [u8; 16380],  // we allow 16kb info per frame (we deduct 2 + 1 + 1 meta info)
}

impl Frame {
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.data[0..(self.size as usize)]).into_owned()
    }
}

pub fn create_frame_from_bytes(buffer: [u8; 16384]) -> Frame {
    unsafe { std::mem::transmute::<[u8; 16384], Frame>(buffer) }
}

#[cfg(test)]
mod tests {
    use super::create_frame_from_bytes;

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
}
