use glutin::{
    self,
    config::ConfigTemplateBuilder,
    context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext},
    display::{Display, DisplayApiPreference},
    prelude::*,
    surface::{Surface, WindowSurface},
};
use glow::HasContext;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;
use std::ffi::CString;
use std::fs;
use std::path::Path;


pub struct RenderManager {
    gl: glow::Context,
    surface: Surface<WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
	shader_program: glow::Program,
	vao: glow::VertexArray,
	start_time: std::time::Instant,
}

impl RenderManager {
    fn get_asset_path(relative_path: &str) -> PathBuf {
        let base_dir = if cfg!(debug_assertions) {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        } else {
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf()
        };
        
        return base_dir.join(relative_path)
    }

    fn load_shader(shader_path: &str) -> String {
        let shader_path = Self::get_asset_path(shader_path);
        fs::read_to_string(&shader_path)
            .unwrap_or_else(|_| panic!("Failed to read shader file: {}", shader_path.display()))
    }
	fn compile_shader(gl: &glow::Context, source: &str, shader_type: u32) -> glow::Shader {
        unsafe {
            let shader = gl.create_shader(shader_type).expect("Cannot create shader");
            gl.shader_source(shader, source);
            gl.compile_shader(shader);

            if !gl.get_shader_compile_status(shader) {
                panic!("Failed to compile shader: {}", gl.get_shader_info_log(shader));
            }
            shader
        }
    }

    fn create_shader_program(gl: &glow::Context, vertex_shader: glow::Shader, fragment_shader: glow::Shader) -> glow::Program {
        unsafe {
            let program = gl.create_program().expect("Cannot create program");
            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                panic!("Failed to link program: {}", gl.get_program_info_log(program));
            }

            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);
            program
        }
    }
    pub fn new(window: &Window) -> Self {
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(true)
            .build();

        // Create display
        let display = unsafe {
            Display::new(
                window.display_handle()
                    .map_err(|e| e.to_string())
                    .unwrap()
                    .as_raw(),
                DisplayApiPreference::Wgl(None)
            )
            .expect("Failed to create display")
        };

        let config = unsafe {
            display
                .find_configs(template)
                .expect("Failed to find configs")
                .next()
                .expect("No config found")
        };

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(None))
            .build(Some(
                window.window_handle()
                    .map_err(|e| e.to_string())
                    .unwrap()
                    .as_raw()
            ));

        let context = unsafe {
            display
                .create_context(&config, &context_attributes)
                .expect("Failed to create context")
        };

        let size = window.inner_size();
        let surface_attributes = 
            glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new().build(
                window.window_handle()
                    .map_err(|e| e.to_string())
                    .unwrap()
                    .as_raw(),
                std::num::NonZeroU32::new(size.width).unwrap(),
                std::num::NonZeroU32::new(size.height).unwrap(),
            );

        let surface = unsafe {
            display
                .create_window_surface(&config, &surface_attributes)
                .expect("Failed to create surface")
        };

        let context = context
            .make_current(&surface)
            .expect("Failed to make context current");

        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                let s = CString::new(s).unwrap();
                display.get_proc_address(s.as_c_str()) as *const _
            })
        };

		let vertices: [f32; 12] = [
			-1.0, 1.0, // top left
			-1.0, -1.0, // bottom left
			1.0, -1.0, // bottom right
			-1.0, 1.0, // top left
			1.0, -1.0, // bottom right
			1.0, 1.0, // top right
		];

        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            let vbo = gl.create_buffer().unwrap();
            
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    vertices.as_ptr() as *const u8,
                    vertices.len() * std::mem::size_of::<f32>(),
                ),
                glow::STATIC_DRAW,
            );

            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 2 * std::mem::size_of::<f32>() as i32, 0);
            gl.enable_vertex_attrib_array(0);

            let vertex_source = Self::load_shader("shaders/vertexshader.glsl");
            let fragment_source = Self::load_shader("shaders/fragmentshader.glsl");
            // Create shader program first
            let vertex_shader = Self::compile_shader(&gl, &vertex_source, glow::VERTEX_SHADER);
            let fragment_shader = Self::compile_shader(&gl, &fragment_source, glow::FRAGMENT_SHADER);
            let shader_program = Self::create_shader_program(&gl, vertex_shader, fragment_shader);

            // Create instance with actual shader program
            Self {
                gl,
                surface,
                context,
                shader_program,
                vao,
                start_time: std::time::Instant::now(),
            }
		}
    }

    pub fn render(&self, size: (u32, u32), mouse: (f64, f64), scroll: f64) {
        unsafe {
           
            self.gl.viewport(0, 0, size.0 as i32, size.1 as i32);
            
            self.gl.use_program(Some(self.shader_program));
            
            // Set time uniform
            let time = self.start_time.elapsed().as_secs_f32();
            let time_location = self.gl.get_uniform_location(self.shader_program, "u_time");
            self.gl.uniform_1_f32(time_location.as_ref(), time);
            
            let resolution_location = self.gl.get_uniform_location(self.shader_program, "u_resolution");
            self.gl.uniform_2_f32(
                resolution_location.as_ref(),
                size.0 as f32,
                size.1 as f32,
            );

            let mouse_location = self.gl.get_uniform_location(self.shader_program, "u_mouse");
            self.gl.uniform_2_f32(
                mouse_location.as_ref(),
                mouse.0 as f32,
                mouse.1 as f32,
            );

            let scroll_location = self.gl.get_uniform_location(self.shader_program, "u_scroll");
            self.gl.uniform_1_f32(
                scroll_location.as_ref(),
                scroll as f32
            );

            // Clear and draw
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
            
            self.surface.swap_buffers(&self.context).unwrap();
        }
    }
}