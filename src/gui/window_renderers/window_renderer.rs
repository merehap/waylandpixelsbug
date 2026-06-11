use egui::{Context, Ui};
use winit::dpi::Position;

pub trait WindowRenderer {
    fn ui(&mut self, ctx: &Context, ui: &mut Ui) -> FlowControl;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

pub type WindowArgs = (Box<dyn WindowRenderer>, Position, u64);

pub struct FlowControl {
    pub window_args: Option<WindowArgs>,
    pub should_close_window: bool,
}

impl FlowControl {
    pub const CONTINUE: Self = Self { window_args: None, should_close_window: false };

    pub fn spawn_window(window_args: WindowArgs) -> Self {
        Self {
            window_args: Some(window_args),
            should_close_window: false,
        }
    }
}