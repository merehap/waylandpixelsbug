use egui::{Context, Ui};

use crate::gui::window_renderers::window_renderer::WindowRenderer;

pub struct StatusRenderer;

impl WindowRenderer for StatusRenderer {
    fn ui(&mut self, _ctx: &Context, _ui: &mut Ui) -> Option<Box<dyn WindowRenderer>> {
        None
    }
}