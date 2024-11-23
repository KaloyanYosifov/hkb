use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};
use thiserror::Error as ThisError;

#[derive(Debug)]
struct JpegHeader {
    identifer: String,
    version: String,
    units: u8,
    density: String,
    thumbnail: String,
}

impl JpegHeader {
    fn from_reader(reader: &mut BufReader<File>) -> Option<Self> {
        let mut bytes: [u8; 16] = [0; 16];
        reader.read_exact(&mut bytes).unwrap();

        let length = u16::from_be_bytes(bytes[0..2].try_into().unwrap());

        // if our length is 16
        // we got our JFIF Jpeg
        // otherwise we have a thumbnail header
        // which we do not support for now
        if length == 16 {
            let identifer = String::from_utf8_lossy(&bytes[2..6]).to_string();
            let version = format!("{}.{}", bytes[7], bytes[8]);
            let units = bytes[7];

            let x_density = u16::from_be_bytes(bytes[10..12].try_into().unwrap());
            let y_density = u16::from_be_bytes(bytes[12..14].try_into().unwrap());
            let density = format!("{}x{}", x_density, y_density);

            let thumbnail = format!("{}x{}", bytes[14], bytes[15]);

            Some(Self {
                identifer,
                units,
                version,
                density,
                thumbnail,
            })
        } else {
            None
        }
    }
}

const JPEG_START_MARKER: [u8; 2] = [255, 216];
const JPEG_HEADER_MARKER: [u8; 2] = [255, 224];
const JPEG_QUANTIZATION_MARKER: [u8; 2] = [255, 219];
const JPEG_FRAME_START_MARKER: [u8; 2] = [255, 192];
const JPEG_HUFFMAN_MARKER: [u8; 2] = [255, 196];
const JPEG_END_MARKER: [u8; 2] = [255, 217];

fn bytes_match(bytes: &[u8; 2], bytes2: &[u8; 2]) -> bool {
    bytes[0] == bytes2[0] && bytes[1] == bytes2[1]
}

enum JpegMarker {
    Start,
    Header,
    End,
    QuantizationTable,
    FrameStart,
    HuffmanTable,
}

impl JpegMarker {
    fn get_marker(bytes: &[u8; 2]) -> Option<Self> {
        if bytes_match(bytes, &JPEG_START_MARKER) {
            Some(Self::Start)
        } else if bytes_match(bytes, &JPEG_END_MARKER) {
            Some(Self::End)
        } else if bytes_match(bytes, &JPEG_HEADER_MARKER) {
            Some(Self::Header)
        } else if bytes_match(bytes, &JPEG_QUANTIZATION_MARKER) {
            Some(Self::QuantizationTable)
        } else if bytes_match(bytes, &JPEG_FRAME_START_MARKER) {
            Some(Self::FrameStart)
        } else if bytes_match(bytes, &JPEG_HUFFMAN_MARKER) {
            Some(Self::HuffmanTable)
        } else {
            None
        }
    }
}

#[derive(ThisError, Debug)]
pub enum JpegDecoderError {
    #[error("File does not exist!")]
    FileDoesNotExist,

    #[error("File cannot be opened!")]
    FileCannotBeOpened(#[from] std::io::Error),

    #[error("File is not a jpeg image")]
    NotAJpegFile,
}

struct JpegDecoder;

impl JpegDecoder {
    pub fn new() -> Self {
        Self {}
    }
}

impl JpegDecoder {
    pub fn decode(&self, path: impl AsRef<str>) -> Result<(), JpegDecoderError> {
        let path = Path::new(path.as_ref());

        if !path.exists() {
            return Err(JpegDecoderError::FileDoesNotExist);
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut buffer: [u8; 2] = [0; 2];
        reader.read_exact(&mut buffer)?;

        let marker = JpegMarker::get_marker(&buffer);

        if marker.is_none() || !matches!(marker.unwrap(), JpegMarker::Start) {
            return Err(JpegDecoderError::NotAJpegFile);
        }

        loop {
            let result = reader.read_exact(&mut buffer);

            if result.is_err() {
                break;
            }

            let marker = JpegMarker::get_marker(&buffer);

            if let Some(actual_marker) = marker {
                match actual_marker {
                    JpegMarker::Header => {
                        println!("{:?}", JpegHeader::from_reader(&mut reader));
                    }
                    JpegMarker::QuantizationTable => {
                        // TODO: implement a vector containing tables
                        let mut bytes: [u8; 2] = [0; 2];
                        reader.read_exact(&mut bytes)?;

                        let length = u16::from_be_bytes(bytes[0..2].try_into().unwrap());
                        let mut bytes = vec![0; length as usize];
                        reader.read_exact(&mut bytes)?;
                        println!("QuantizationTable: {:?}", bytes);
                    }
                    JpegMarker::FrameStart => {
                        // TODO: implement a vector containing tables
                        let mut bytes: [u8; 2] = [0; 2];
                        reader.read_exact(&mut bytes)?;

                        let length = u16::from_be_bytes(bytes[0..2].try_into().unwrap());
                        let mut bytes = vec![0; length as usize];
                        reader.read_exact(&mut bytes)?;
                        println!("FrameStart: {:?}", bytes);
                    }
                    JpegMarker::HuffmanTable => {
                        // TODO: implement a vector containing tables
                        let mut bytes: [u8; 2] = [0; 2];
                        reader.read_exact(&mut bytes)?;

                        let length = u16::from_be_bytes(bytes[0..2].try_into().unwrap());
                        let mut bytes = vec![0; length as usize];
                        reader.read_exact(&mut bytes)?;
                        println!("HuffmanTable: {:?}", bytes);
                    }
                    JpegMarker::End => break,
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn it_can_open_jpeg_file() {
        let decoder = JpegDecoder::new();

        decoder.decode("./test-files/image.jpeg").unwrap();

        assert!(false);
    }

    #[ignore]
    #[test]
    fn it_can_open_big_jpeg_file() {
        let decoder = JpegDecoder::new();

        decoder.decode("./test-files/big_4k.jpg").unwrap();

        assert!(false);
    }
}
