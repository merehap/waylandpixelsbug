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

use crate::gui::window_renderers::window_renderer::{WindowRenderer, FlowControl};
use crate::gui::window_renderers::primary_renderer::PrimaryRenderer;

const PRIMARY_WINDOW_SCALE_FACTOR: f32 = 3.0;

pub struct EguiGui<'a> {
    window_manager: WindowManager<'a>,
}

impl <'a> EguiGui<'a> {
    pub fn new() -> Self {
        Self {
            window_manager: WindowManager::new(),
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.run_app(self).unwrap();
    }
}

impl <'a> ApplicationHandler for EguiGui<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let primary_renderer = Box::new(PrimaryRenderer);
        let position = Position::Physical(PhysicalPosition { x: 50, y: 50 });
        self.window_manager.create_window_from_renderer(event_loop, primary_renderer, position, PRIMARY_WINDOW_SCALE_FACTOR as f64);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.window_manager.request_redraws();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.window_manager.remove_window(window_id);
            }
            WindowEvent::RedrawRequested => {
                match self.window_manager.draw(window_id) {
                    Ok(FlowControl { window_args, should_close_window }) => {
                        if let Some((renderer, position, scale)) = window_args {
                            self.window_manager.create_window_from_renderer(
                                event_loop,
                                renderer,
                                position,
                                scale as f64,
                            );
                        }

                        if should_close_window {
                            self.window_manager.remove_window(window_id);
                        }
                    }
                    Err(e) => {
                        println!("Closing due to redraw failure. {e}");
                        event_loop.exit();
                    }
                }
            }
            _ => {
                if let Some(window) = self.window_manager.window_mut(window_id) {
                    window.handle_event(&event);
                }
            }
        }
    }
}

/// Manages all state required for rendering egui over `Pixels`.
struct EguiWindow<'a> {
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    wgpu_renderer: Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,
    has_presented_frame: bool,

    // State for the GUI
    window: Arc<Window>,
    pixels: Pixels<'a>,
    window_renderer: Box<dyn WindowRenderer>,
}

impl<'a> EguiWindow<'a> {
    fn from_active_event_loop(
        event_loop: &ActiveEventLoop,
        scale_factor: f64,
        initial_position: Position,
        renderer: Box<dyn WindowRenderer>,
    ) -> Self {
        let window = {
            let size = LogicalSize::new(
                scale_factor * renderer.width() as f64,
                scale_factor * renderer.height() as f64,
            );
            let window_attributes = Window::default_attributes()
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_resizable(false)
                .with_visible(false)
                .with_position(initial_position);
            event_loop.create_window(window_attributes).unwrap()
        };

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let window = Arc::new(window);
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.clone());
        let pixels = Pixels::new(
            renderer.width() as u32,
            renderer.height() as u32,
            surface_texture,
        )
        .unwrap();

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
        pixels: pixels::Pixels<'a>,
        window_renderer: Box<dyn WindowRenderer>,
    ) -> Self {
        let egui_ctx = Context::default();
        egui_extras::install_image_loaders(&egui_ctx);
        let egui_state = egui_winit::State::new(
            egui_ctx,
            ViewportId::ROOT,
            &window,
            None,
            None,
            Some(pixels.device().limits().max_texture_dimension_2d as usize),
        );
        let screen_descriptor = ScreenDescriptor {
            pixels_per_point: scale_factor,
            size_in_pixels: [width, height],
        };
        let renderer_options = RendererOptions::default();
        let wgpu_renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), renderer_options);
        let textures = TexturesDelta::default();

        Self {
            egui_state,
            screen_descriptor,
            wgpu_renderer,
            paint_jobs: Vec::new(),
            textures,
            has_presented_frame: false,
            window,
            pixels,
            window_renderer,
        }
    }

    /// Handle input events from the window manager.
    fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        let _event_response = self.egui_state.on_window_event(&self.window, event);
    }

    fn draw(&mut self) -> Result<FlowControl, String> {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let mut raw_input = self.egui_state.take_egui_input(&self.window);

        raw_input.viewports.iter_mut().for_each(|viewport| {
            // Hack around bug with scale factor causing egui to crash in fonts lookup
            viewport.1.native_pixels_per_point = Some(PRIMARY_WINDOW_SCALE_FACTOR);
        });

        let mut result = FlowControl::CONTINUE;
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
            if !self.has_presented_frame {
                self.window.set_visible(true);
                self.has_presented_frame = true;
            }

        Ok(result)
    }
}

struct WindowManager<'a> {
    windows_by_id: BTreeMap<WindowId, EguiWindow<'a>>,
}

impl<'a> WindowManager<'a> {
    pub fn new() -> WindowManager<'a> {
        WindowManager {
            windows_by_id: BTreeMap::new(),
        }
    }

    pub fn create_window_from_renderer(
        &mut self,
        event_loop: &ActiveEventLoop,
        renderer: Box<dyn WindowRenderer>,
        position: Position,
        scale: f64,
    ) {
        let window = EguiWindow::from_active_event_loop(event_loop, scale, position, renderer);
        self.windows_by_id.insert(window.window.id(), window);
    }

    pub fn remove_window(&mut self, window_id: WindowId) {
        self.windows_by_id.remove(&window_id);
    }

    pub fn request_redraws(&self) {
        for window in self.windows_by_id.values() {
            window.window.request_redraw();
        }
    }

    pub fn draw(
        &mut self,
        window_id: WindowId,
    ) -> Result<FlowControl, String> {
        let window = self
            .window_mut(window_id)
            .ok_or("Failed to create window")?;
        window.draw()
    }

    pub fn window_mut(&mut self, window_id: WindowId) -> Option<&mut EguiWindow<'a>> {
        self.windows_by_id.get_mut(&window_id)
    }
}