use egui::{Context, Ui};
pub use winit::dpi::{PhysicalPosition, Position};

use crate::gui::window_renderers::window_renderer::{WindowRenderer, FlowControl};
use crate::gui::window_renderers::status_renderer::StatusRenderer;

pub struct PrimaryRenderer;

impl WindowRenderer for PrimaryRenderer {
    fn ui(&mut self, _ctx: &Context, ui: &mut Ui) -> FlowControl {
        let mut result = FlowControl::CONTINUE;
        egui::Panel::top("menubar_container").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Debug Windows", |ui| {
                    if ui.button("Status").clicked() {
                        ui.close();
                        result = FlowControl::spawn_window((
                            Box::new(StatusRenderer) as Box<dyn WindowRenderer>,
                            Position::Physical(PhysicalPosition { x: 850, y: 360 }),
                            2,
                        ));
                    }
                })
            });
        });

        result
    }

    fn width(&self) -> usize {
        200
    }

    fn height(&self) -> usize {
        200
    }
}