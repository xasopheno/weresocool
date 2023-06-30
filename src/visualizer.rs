use wgpu;

use shaderc::Compiler;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use weresocool_instrument::StereoWaveform;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn start_visualizer() {
    // let (tx, rx) = mpsc::channel();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Audio Visualizer")
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
    });
    let surface = unsafe { instance.create_surface(&window).unwrap() };

    // Create a new VisualizerState
    let visualizer_state = Arc::new(Mutex::new(VisualizerState::new(
        &event_loop,
        &surface,
        window,
        &instance,
    )));
    let visualizer_state_clone = Arc::clone(&visualizer_state);

    // let visualizer_state_clone = Arc::clone(&visualizer_state);
    // thread::spawn(move || {
    // while let Ok(audio_buffer) = rx.recv() {
    // visualizer_state_clone
    // .lock()
    // .unwrap()
    // .visualize_rendered_buffer(audio_buffer);
    // }
    // });
    // Create a new event loop
    // Create a channel for sending audio buffers to the visualizer thread

    event_loop.run(move |event, _window_target, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
                dbg!("redraw");
                visualizer_state_clone
                    .lock()
                    .unwrap()
                    .visualize_rendered_buffer();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                // Handle window resize here
            }
            _ => (),
        }

        // Request redraw from the window associated with the visualizer state
        visualizer_state.lock().unwrap().window.request_redraw();
    });
}

struct VisualizerState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: winit::window::Window,
    render_pipeline: wgpu::RenderPipeline,
    texture: wgpu::Texture,
    color_target: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

impl VisualizerState {
    fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        surface: &wgpu::Surface,
        window: winit::window::Window,
        instance: &wgpu::Instance,
    ) -> VisualizerState {
        let (device, queue) = futures::executor::block_on(async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap();

            adapter
                .request_device(&wgpu::DeviceDescriptor::default(), None)
                .await
                .unwrap()
        });
        let texture_size = wgpu::Extent3d {
            width: 1024,
            height: 1024,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Spectrum Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let color_target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Color Target"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let (render_pipeline, bind_group) =
            VisualizerState::create_render_pipeline(&device, &texture);

        VisualizerState {
            device,
            queue,
            window,
            render_pipeline,
            texture,
            color_target,
            bind_group,
        }
    }

    // fn visualize_rendered_buffer(&self, audio_buffer: Arc<StereoWaveform>) {
    fn visualize_rendered_buffer(&self) {
        let spectrum = self.to_spectrum();

        self.create_texture_from_spectrum(&spectrum);

        self.draw_texture_to_window();
    }

    fn create_texture_from_spectrum(&self, spectrum: &Vec<f32>) {
        let texture_size = wgpu::Extent3d {
            width: 1024,
            height: 1024,
            depth_or_array_layers: 1,
        };

        let texture_data = spectrum
            .iter()
            .map(|&x| (x * 255.0) as u8)
            .chain(std::iter::repeat(1))
            .take((1024) as usize)
            .collect::<Vec<_>>();

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(texture_size.width * 4),
                rows_per_image: Some(texture_size.height),
            },
            texture_size,
        );
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        texture: &wgpu::Texture,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Texture Bind Group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout], // include your bind group layout here
            push_constant_ranges: &[],
        });

        let vertex_shader_data =
            std::fs::read_to_string("./src/shader.vert").expect("Failed to read vertex shader");
        let compiler = Compiler::new().unwrap();

        let vertex_shader = compiler
            .compile_into_spirv(
                &vertex_shader_data,
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap_or_else(|e| panic!("Failed to compile vertext shader: {}", e));
        let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::util::make_spirv(&vertex_shader.as_binary_u8()),
        });

        let fragment_shader_data =
            std::fs::read_to_string("./src/shader.frag").expect("Failed to read fragment shader");
        let fragment_shader = compiler
            .compile_into_spirv(
                &fragment_shader_data,
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None,
            )
            .unwrap_or_else(|e| panic!("Failed to compile fragment shader: {}", e));
        dbg!("here");
        let fragment_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::util::make_spirv(&fragment_shader.as_binary_u8()),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            multiview: None,
            vertex: wgpu::VertexState {
                module: &vertex_shader_module,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader_module,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        (render_pipeline, bind_group)
    }

    fn draw_texture_to_window(&self) {
        let texture_view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let color_view = self
            .color_target
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view, // Use color_view instead of texture_view
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]); // Set the bind group
            render_pass.draw(0..4, 0..1);
        }

        let command_buffer = command_encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));
    }
    fn to_spectrum(&self) -> Vec<f32> {
        let mut data = vec![0.0; 1024];
        for (i, item) in data.iter_mut().enumerate() {
            *item = ((i as f32) / 128.0).sin();
        }
        data
        // fn to_spectrum(&self, audio_buffer: Arc<StereoWaveform>) -> Vec<f32> {
        // TODO: Implement the details of the FFT transformation
        // Here is a simplified example:

        // let mut planner = rustfft::FftPlanner::new();
        // let fft = planner.plan_fft_forward(audio_buffer.l_buffer.len());

        // let mut complex = audio_buffer
        // .l_buffer
        // .iter()
        // .map(|&x| rustfft::num_complex::Complex {
        // re: x as f32,
        // im: 0.0,
        // })
        // .collect::<Vec<_>>();

        // fft.process(&mut complex);

        // complex.iter().map(|x| x.norm()).collect::<Vec<f32>>()
        //
        // vec![]
    }
}
