pub use input::*;
pub use navigation::*;
use ratatui::{prelude::Rect, Frame};

mod input;
mod navigation;

pub trait Component {
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

pub trait StatefulComponent {
    type State;

    fn render(&mut self, frame: &mut Frame, state: &mut Self::State, area: Rect);
}
