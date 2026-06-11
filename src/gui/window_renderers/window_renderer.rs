use egui::{Context, Ui};

pub trait WindowRenderer {
    fn ui(&mut self, ctx: &Context, ui: &mut Ui) -> Option<Box<dyn WindowRenderer>>;
}