use std::collections::BTreeMap;
use std::sync::Arc;

use egui::{ClippedPrimitive, Context, TexturesDelta, ViewportId};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use pixels::{Pixels, SurfaceTexture};
use pixels::wgpu::{RenderPassDescriptor, RenderPassColorAttachment, Operations, LoadOp, StoreOp};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, Position};
use winit::event::WindowEvent;
use winit::event_loop::{EventLoop, ActiveEventLoop};
use winit::window::{Window, WindowId};

use crate::gui::window_renderers::window_renderer::WindowRenderer;
use crate::gui::window_renderers::primary_renderer::PrimaryRenderer;

const PRIMARY_WINDOW_SCALE_FACTOR: f32 = 1.0;
const WINDOW_DIMENSION: u32 = 500;

pub struct EguiGui {
    windows_by_id: BTreeMap<WindowId, EguiWindow>,
}

impl EguiGui {
    pub fn new() -> Self {
        Self {
            windows_by_id: BTreeMap::new(),
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.run_app(self).unwrap();
    }

    fn create_window_from_renderer(
        &mut self,
        event_loop: &ActiveEventLoop,
        renderer: Box<dyn WindowRenderer>,
        position: Position,
        scale: f64,
    ) {
        let window = EguiWindow::from_active_event_loop(event_loop, scale, position, renderer);
        self.windows_by_id.insert(window.window.id(), window);
    }

    fn request_redraws(&self) {
        for window in self.windows_by_id.values() {
            window.window.request_redraw();
        }
    }

    fn draw(&mut self, window_id: WindowId) -> Result<Option<Box<dyn WindowRenderer>>, String> {
        let window = self.windows_by_id.get_mut(&window_id)
            .ok_or("Failed to create window")?;
        window.draw()
    }
}

impl ApplicationHandler for EguiGui {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let primary_renderer = Box::new(PrimaryRenderer);
        let position = Position::Physical(PhysicalPosition { x: 50, y: 50 });
        self.create_window_from_renderer(event_loop, primary_renderer, position, PRIMARY_WINDOW_SCALE_FACTOR as f64);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.request_redraws();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.windows_by_id.remove(&window_id);
            }
            WindowEvent::RedrawRequested => {
                match self.draw(window_id) {
                    Ok(renderer) => {
                        if let Some(renderer) = renderer {
                            self.create_window_from_renderer(
                                event_loop,
                                renderer,
                                Position::Physical(PhysicalPosition { x: 100, y: 100 }),
                                1.0,
                            );
                        }
                    }
                    Err(e) => {
                        println!("Closing due to redraw failure. {e}");
                        event_loop.exit();
                    }
                }
            }
            _ => {
                if let Some(window) = self.windows_by_id.get_mut(&window_id) {
                    window.handle_event(&event);
                }
            }
        }
    }
}

/// Manages all state required for rendering egui over `Pixels`.
struct EguiWindow {
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    wgpu_renderer: Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,

    // State for the GUI
    window: Arc<Window>,
    pixels: Pixels<'static>,
    window_renderer: Box<dyn WindowRenderer>,
}

impl EguiWindow {
    fn from_active_event_loop(
        event_loop: &ActiveEventLoop,
        scale_factor: f64,
        initial_position: Position,
        renderer: Box<dyn WindowRenderer>,
    ) -> Self {
        let window = {
            let size = LogicalSize::new(scale_factor * WINDOW_DIMENSION as f64, scale_factor * WINDOW_DIMENSION as f64);
            let window_attributes = Window::default_attributes()
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_resizable(false)
                .with_visible(true)
                .with_position(initial_position);
            event_loop.create_window(window_attributes).unwrap()
        };

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let window = Arc::new(window);
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.clone());
        let pixels = Pixels::new(WINDOW_DIMENSION, WINDOW_DIMENSION, surface_texture).unwrap();

        EguiWindow::new(
            window_size.width,
            window_size.height,
            scale_factor,
            window.clone(),
            pixels,
            renderer,
        )
    }

    fn new(
        width: u32,
        height: u32,
        scale_factor: f32,
        window: Arc<Window>,
        pixels: pixels::Pixels<'static>,
        window_renderer: Box<dyn WindowRenderer>,
    ) -> Self {
        let egui_state = egui_winit::State::new(
            Context::default(),
            ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );
        let screen_descriptor = ScreenDescriptor {
            pixels_per_point: scale_factor,
            size_in_pixels: [width, height],
        };
        let renderer_options = RendererOptions::default();
        let wgpu_renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), renderer_options);
        Self {
            egui_state,
            screen_descriptor,
            wgpu_renderer,
            paint_jobs: Vec::new(),
            textures: TexturesDelta::default(),
            window,
            pixels,
            window_renderer,
        }
    }

    /// Handle input events from the window manager.
    fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        let _event_response = self.egui_state.on_window_event(&self.window, event);
    }

    fn draw(&mut self) -> Result<Option<Box<dyn WindowRenderer>>, String> {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(&self.window);

        let mut result = None;
        let output = self.egui_state.egui_ctx().run_ui(raw_input, |ui| {
            result = self.window_renderer.ui(self.egui_state.egui_ctx(), ui);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(&self.window, output.platform_output);
        self.paint_jobs = self
            .egui_state
            .egui_ctx()
            .tessellate(output.shapes, PRIMARY_WINDOW_SCALE_FACTOR);

        self.pixels
            .render_with(|encoder, render_target, context| {
                context.scaling_renderer.render(encoder, render_target);
                for (id, delta) in &self.textures.set {
                    self.wgpu_renderer.update_texture(&context.device, &context.queue, *id, delta);
                }
                self.wgpu_renderer.update_buffers(
                    &context.device,
                    &context.queue,
                    encoder,
                    &self.paint_jobs,
                    &self.screen_descriptor,
                );

                {
                    let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("egui"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: render_target,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Load,
                                store: StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    }).forget_lifetime();

                    // Record all render passes.
                    self.wgpu_renderer.render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
                }

                // Cleanup
                let textures = std::mem::take(&mut self.textures);
                for id in &textures.free {
                    self.wgpu_renderer.free_texture(id);
                }

                Ok(())
            })
            .map_err(|err| err.to_string())?;

        Ok(result)
    }
}