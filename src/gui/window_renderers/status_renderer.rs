use egui::{Context, Ui};

use crate::gui::window_renderers::window_renderer::{FlowControl, WindowRenderer};

pub struct StatusRenderer;

impl StatusRenderer {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;
}

impl WindowRenderer for StatusRenderer {
    fn ui(&mut self, _ctx: &Context, _ui: &mut Ui) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}