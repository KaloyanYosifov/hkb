use crate::components::Image;
use hkb_core::logger::debug;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use ratatui::prelude::{Frame, Rect};

pub struct MainApp {
    image: DynamicImage,
}

impl MainApp {
    pub fn new() -> Self {
        let image = ImageReader::open("images/image.jpeg")
            .unwrap()
            .decode()
            .unwrap();

        Self { image }
    }
}

impl MainApp {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let area_width = area.width as u32;
        let area_height = area.height as u32;

        if self.image.width() > area_width || self.image.height() > area_height {
            self.image = self.image.resize(
                area_width,
                area_height,
                image::imageops::FilterType::Nearest,
            );

            debug!(target: "CLIENT_MAIN", "Resized image to {}x{}", area_width, area_height);
        }

        frame.render_widget(Image::new(&self.image), area)
    }
}
