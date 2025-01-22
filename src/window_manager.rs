use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use crate::render_manager::RenderManager;

fn map(value: f64, start1: f64, stop1: f64, start2: f64, stop2: f64) -> f64 {
    return start2 + (stop2 - start2) * ((value - start1) / (stop1 - start1));
}

pub struct App {
    window: Option<Window>,
    render_manager: Option<RenderManager>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            render_manager: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Fractal Window")
            .with_maximized(true);
        let window = event_loop.create_window(window_attributes).unwrap();
        self.render_manager = Some(RenderManager::new(&window));
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        static mut CURSOR: (f64, f64) = (0.0, 0.0);
        static mut ZOOM: f64 = 1.0;
        static mut POS: (f64, f64) = (0.0, 0.0);
        
        let size = {
            let window = self.window.as_ref().unwrap();
            let size = window.inner_size();
            (size.width, size.height)
        };
        
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                if let Some(render_manager) = &self.render_manager {
                    
                    render_manager.render(size, unsafe { POS }, unsafe {ZOOM});
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            WindowEvent::MouseWheel { device_id:_, delta, phase:_} => {
                unsafe {
                    let increment = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => y as f64,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y,
                    };
                    
                    let tempx = map(CURSOR.0, 0.0, size.0 as f64, -2.0, 2.0) * ZOOM + POS.0;
                    let tempy = map(CURSOR.1, 0.0, size.1 as f64, 2.0, -2.0) * ZOOM + POS.1;
                    
                    if increment < 0.0 {
                        ZOOM *= 1.1;
                    } else {
                        ZOOM *= 0.9;
                    }
                    POS.0 = tempx - map(CURSOR.0, 0.0, size.0 as f64, -2.0, 2.0) * ZOOM;
                    POS.1 = tempy - map(CURSOR.1, 0.0, size.1 as f64, 2.0, -2.0) * ZOOM;
                }
            },
            WindowEvent::CursorMoved { device_id: _, position } => {
                unsafe {
                    CURSOR = (position.x, position.y);
                }
            }
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _} => {
                if event.state == ElementState::Pressed {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => {
                            event_loop.exit();
                        }
                        PhysicalKey::Code(KeyCode::KeyW) => {
                            unsafe {
                                POS.1 += 0.1 * ZOOM;
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyS) => {
                            unsafe {
                                POS.1 -= 0.1 * ZOOM;
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyA) => {
                            unsafe {
                                POS.0 -= 0.1 * ZOOM;
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyD) => {
                            unsafe {
                                POS.0 += 0.1 * ZOOM;
                            }
                        }
                        _ => (),
                    }
                }

            }
            _ => (),
        }
    }
}