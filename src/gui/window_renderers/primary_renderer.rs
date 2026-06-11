use egui::{Context, Ui};

use crate::gui::window_renderers::window_renderer::WindowRenderer;
use crate::gui::window_renderers::status_renderer::StatusRenderer;

pub struct PrimaryRenderer;

impl WindowRenderer for PrimaryRenderer {
    fn ui(&mut self, _ctx: &Context, ui: &mut Ui) -> Option<Box<dyn WindowRenderer>> {
        let mut result = None;
        egui::Panel::top("menubar_container").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Debug Windows", |ui| {
                    if ui.button("Status").clicked() {
                        ui.close();
                        result = Some(Box::new(StatusRenderer) as Box<dyn WindowRenderer>);
                    }
                })
            });
        });

        result
    }
}