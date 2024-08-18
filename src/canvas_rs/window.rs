// manages a single window, providing GPU behind a convenient interface
// this is really just a pathway to WGPU with Winit, but it's designed to be somewhat modular.
// in the future, we'll want to implement the Bevy plugin interface so we can use
// our window's event loop as a runner.

pub struct CanvasProperties {
    pub resizable : bool,
    pub width : u32,
    pub height : u32
}


impl CanvasProperties {
    pub fn default() -> Self {
        Self {
            resizable : true,
            width : 500,
            height : 500
        }
    }
}


pub enum Event {
    FrameUpdate
}


pub trait Canvas { // the actual window trait
    // this MUST be the event loop driver
    // this is because underlying graphics code gets mad if it doesn't get to steer the event loop
    // provides a context to the event function
    fn set_properties(&mut self, properties : CanvasProperties);

    fn event_loop(&mut self, evt : fn(Event) -> (), draw : fn(&mut dyn Context2d) -> ()) -> Result<(), Box<dyn std::error::Error>>;
}


pub struct Color {
    pub r : u8, // red (0-255)
    pub g : u8, // green (0-255)
    pub b : u8, // blue (0-255)
    pub a : f32 // alpha (0.0-1.0)
}


pub struct StrokeParameters {
    pub width : f32,
    pub color : Color
}


pub struct FillParameters {
    pub color : Color
}


pub struct Transform {
    matrix : [[f32; 2]; 2] // 2x2 transform matrix
}


impl Transform {
    fn new() -> Self {
        Self {
            matrix : [
                [1.0, 0.0],
                [0.0, 1.0]
            ]
        }
    }
}


pub trait Context2d { // the 2d context
    // does immediate mode drawing

    // drawing paths
    fn path_moveto(&mut self, x : f32, y : f32); // move the "head" to a position WITHOUT drawing

    fn path_lineto(&mut self, x : f32, y : f32); // move the "head" to a position and draw the line between the new position and the old position

    // draw the current path buffer (usually means calling the GPU)
    fn path_fill(&mut self); // fill the path

    fn path_stroke(&mut self); // stroke the path
    // this can be done at any time and does NOT flush the buffer, meaning you can construct complex paths and "stamp" them with path_fill and path_stroke

    // clear buffers
    fn path_clearbuf(&mut self); // clears the current path buffer, resets the head to (0, 0)

    // draw shape primitives
    fn fill_rect(&mut self, x : f32, y : f32, w : f32, h : f32);

    fn stroke_rect(&mut self, x : f32, y : f32, w : f32, h : f32);

    fn fill_poly(&mut self, x : f32, y : f32, radius : f32, sides : f32);

    fn stroke_poly(&mut self, x : f32, y : f32, radius : f32, sides : f32);

    // change context parameters
    fn set_stroke_params(&mut self, params : StrokeParameters);

    fn set_fill_params(&mut self, params : FillParameters);

    fn set_stroke_color(&mut self, color : Color);

    fn set_stroke_width(&mut self, width : f32);
    
    fn set_fill_color(&mut self, color : Color);

    // transforms
    fn transform(&mut self) -> &mut Transform; // must return a Transform that can be modified

    // utility
    fn resize(&mut self, w : u32, h : u32);
}


pub struct WgpuContext<'a> { // one of these is constructed every frame for the render program
    // providable by WinitCanvas
    // it literally just wraps a wgpu render pass
    transform : Transform,
    width : u32,
    height : u32,
    render_pass : wgpu::RenderPass<'a>,
    device : &'a wgpu::Device
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Context2d for WgpuContext<'_> {
    // drawing paths
    fn path_moveto(&mut self, x : f32, y : f32) {} // move the "head" to a position WITHOUT drawing

    fn path_lineto(&mut self, x : f32, y : f32) {} // move the "head" to a position and draw the line between the new position and the old position

    // draw the current path buffer (usually means calling the GPU)
    fn path_fill(&mut self) {} // fill the path

    fn path_stroke(&mut self) {} // stroke the path
    // this can be done at any time and does NOT flush the buffer, meaning you can construct complex paths and "stamp" them with path_fill and path_stroke

    // clear buffers
    fn path_clearbuf(&mut self) {} // clears the current path buffer, resets the head to (0, 0)

    // draw shape primitives
    fn fill_rect(&mut self, x : f32, y : f32, w : f32, h : f32) {
        const VERTICES: &[Vertex] = &[
            Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
            Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
            Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
        ];
        self.render_pass.draw(0..VERTICES.len() as u32, 0..1);
    }

    fn stroke_rect(&mut self, x : f32, y : f32, w : f32, h : f32) {}

    fn fill_poly(&mut self, x : f32, y : f32, radius : f32, sides : f32) {}

    fn stroke_poly(&mut self, x : f32, y : f32, radius : f32, sides : f32) {}

    // change context parameters
    fn set_stroke_params(&mut self, params : StrokeParameters) {}

    fn set_fill_params(&mut self, params : FillParameters) {}

    fn set_stroke_color(&mut self, color : Color) {}

    fn set_stroke_width(&mut self, width : f32) {}
    
    fn set_fill_color(&mut self, color : Color) {}

    // transforms
    fn transform(&mut self) -> &mut Transform {
        return &mut self.transform;
    } // must return a Transform that can be modified

    // utility
    fn resize(&mut self, w : u32, h : u32) {
        self.width = w;
        self.height = h;
    }
}

impl<'c> WgpuContext<'c> {
    pub fn new(w : u32, h : u32, pass : wgpu::RenderPass<'c>, device : &'c wgpu::Device) -> WgpuContext<'c> {
        Self {
            transform : Transform::new(),
            width : w,
            height : h,
            render_pass : pass,
            device
        }
    }
}


pub struct WinitCanvas { // use Winit to provide a Canvas
    // the actual winit work is done in a single function, event_loop
    properties : CanvasProperties
}


impl WinitCanvas {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>>{
        Ok(Self {
            properties : CanvasProperties::default()
        })
    }
}

struct VertexBufferManager<'a> {
    device : &'a wgpu::Device,
    buffer : wgpu::Buffer,
    data   : Vec<Vertex>
}

use wgpu::util::DeviceExt;

impl<'c> VertexBufferManager<'c> {
    fn new(device : &'c wgpu::Device) -> Self {
        const STARTING_BUFFER_SIZE : usize = 512;
        let data = vec![Vertex::default(); STARTING_BUFFER_SIZE];
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &[0_u8; STARTING_BUFFER_SIZE],
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        Self {
            device,
            buffer
        }
    }
}


impl Canvas for WinitCanvas {
    fn set_properties(&mut self, properties : CanvasProperties) {
        self.properties = properties;
    }

    fn event_loop(&mut self, evt : fn(Event) -> (), draw : fn(&mut dyn Context2d) -> ()) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = winit::event_loop::EventLoop::new()?;
        let window = winit::window::WindowBuilder::new().build(&event_loop)?;
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends : wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(&window)?;
        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference : wgpu::PowerPreference::default(),
            compatible_surface : Some(&surface),
            force_fallback_adapter : false
        };
        let adapter = futures::executor::block_on(instance.request_adapter(&adapter_options)).unwrap(); // todo: .ok_or(...)
        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features : wgpu::Features::empty(),
                required_limits : wgpu::Limits::default(), // TODO: add WebGL support with downlevel_webgl2_defaults
                label : None
            },
            None
        )).unwrap(); // todo: .ok_or(...)

        // surface capabilities to build a surface configuration
        // this is so dumb
        let scaps = surface.get_capabilities(&adapter);
        let format = scaps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(scaps.formats[0]);
        let properties = CanvasProperties::default();

        let surfaceconfig = wgpu::SurfaceConfiguration {
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width : properties.width,
            height : properties.height,
            present_mode : scaps.present_modes[0],
            alpha_mode : scaps.alpha_modes[0],
            view_formats : vec![],
            desired_maximum_frame_latency : 2
        };
        surface.configure(&device, &surfaceconfig);

        let shader = device.create_shader_module(wgpu::include_wgsl!("main.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label : Some("Render Pipeline Layout"),
            bind_group_layouts : &[],
            push_constant_ranges : &[]
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label : Some("Render Pipeline"),
            layout : Some(&pipeline_layout),
            vertex : wgpu::VertexState {
                module : &shader,
                entry_point : "vertex",
                buffers : &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                    }
                ]
            },
            fragment : Some(wgpu::FragmentState {
                module : &shader,
                entry_point : "fragment",
                targets : &[
                    Some(wgpu::ColorTargetState {
                        format : surfaceconfig.format,
                        blend : Some(wgpu::BlendState::REPLACE),
                        write_mask : wgpu::ColorWrites::ALL
                    })
                ]
            }),
            primitive : wgpu::PrimitiveState {
                topology : wgpu::PrimitiveTopology::TriangleList,
                strip_index_format : None,
                front_face : wgpu::FrontFace::Ccw,
                cull_mode : Some(wgpu::Face::Back),
                polygon_mode : wgpu::PolygonMode::Fill,
                unclipped_depth : false,
                conservative : false
            },
            depth_stencil : None,
            multisample : wgpu::MultisampleState {
                count : 1, 
                mask : !0,
                alpha_to_coverage_enabled : false
            },
            multiview : None
        });

        let vertexbuf = VertexBufferManager::new(&device);

        event_loop.run(|event, control_flow| {
            match event {
                winit::event::Event::WindowEvent {
                    ref event,
                    ..
                } => {
                    match event {
                        winit::event::WindowEvent::CloseRequested => {
                            control_flow.exit();
                        },
                        winit::event::WindowEvent::RedrawRequested => {
                            let output = surface.get_current_texture().unwrap(); // todo: error handling
                            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("Render Encoder"),
                            });
                            { 
                                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Render Pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                                r: 1.0,
                                                g: 1.0,
                                                b: 1.0,
                                                a: 1.0,
                                            }),
                                            store: wgpu::StoreOp::Store,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                    occlusion_query_set: None,
                                    timestamp_writes: None,
                                });
                                render_pass.set_pipeline(&pipeline);
                                let mut ctx = WgpuContext::new(properties.width, properties.height, render_pass, &device);
                                draw(&mut ctx);
                                /*
                                render_pass.draw(0..3, 0..1);*/
                            }

                            // submit will accept anything that implements IntoIter
                            queue.submit(std::iter::once(encoder.finish()));
                            output.present();
                        },
                        _ => {}
                    }
                },
                winit::event::Event::AboutToWait => {
                    window.request_redraw();
                },
                _ => {}
            }
        }).unwrap();
        Ok(())
    }
}
