use hkb_core::logger::debug;
use image::{DynamicImage, GenericImageView, Pixel};
use ratatui::widgets::Widget;

const IMAGE_ALPHABET: [char; 29] = [
    'Ñ', '@', '#', 'W', '$', '9', '8', '7', '6', '5', '4', '3', '2', '1', '0', '?', '!', 'a', 'b',
    'c', ';', ':', '+', '=', '-', ',', '.', '_', ' ',
];

pub struct Image<'a> {
    image: &'a DynamicImage,
}

impl<'a> Image<'a> {
    pub fn new(image: &'a DynamicImage) -> Self {
        Image { image }
    }
}

impl<'a> Widget for Image<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        debug!(target: "CLIENT_COMPONENT_IMAGE", "A {}x{}", area.width, area.height);
        for x in 0..self.image.width() {
            for y in 0..self.image.height() {
                let [red, green, blue] = self.image.get_pixel(x, y).to_rgb().0;
                let rough_estimate = (255_f32 / (IMAGE_ALPHABET.len() as f32)).ceil() as u8;
                let luminance = (0.2126 * (red as f32)
                    + 0.7152 * (green as f32)
                    + 0.0722 * (blue as f32)) as u8;
                let index = luminance / rough_estimate;

                buf.get_mut(area.left() + x as u16, area.top() + y as u16)
                    .set_symbol(IMAGE_ALPHABET[index as usize].to_string().as_str());
            }
        }
    }
}
