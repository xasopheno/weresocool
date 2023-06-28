use crate::{
    manager::RenderManager,
    write::{new_write_output_buffer, write_output_buffer},
};

use weresocool_instrument::StereoWaveform;
use wgpu;

use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencyValue};
use std::cell::RefCell;
use std::cmp::max;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use weresocool_error::Error;
use weresocool_portaudio as pa;
use weresocool_shared::Settings;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn real_time_render_manager(
    render_manager: Arc<Mutex<RenderManager>>,
) -> Result<pa::Stream<pa::NonBlocking, pa::Output<f32>>, Error> {
    let pa = pa::PortAudio::new()?;
    let output_stream_settings = get_output_settings(&pa)?;
    let buffer_size = Settings::global().buffer_size;
    let (tx, rx) = mpsc::channel();
    // Create a new event loop
    thread::spawn(move || {
        // Create a new event loop
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
        // Create a channel for sending audio buffers to the visualizer thread

        // Spawn the visualizer thread
        let visualizer_state_clone = Arc::clone(&visualizer_state);
        thread::spawn(move || {
            while let Ok(audio_buffer) = rx.recv() {
                visualizer_state_clone
                    .lock()
                    .unwrap()
                    .visualize_rendered_buffer(audio_buffer);
            }
        });

        event_loop.run(move |event, _, control_flow| {
            match event {
                // This event is triggered when your window should be redrawn
                Event::RedrawRequested(_) => {
                    // Redraw your window here
                    // visualizer_state.lock().unwrap().draw_texture_to_window([> your texture <]);
                }
                // This event is triggered when your window is closed
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    // End the event loop when the window is closed
                    *control_flow = ControlFlow::Exit;
                }
                // This event is triggered when your window is resized
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    // Handle window resize here
                }
                // Ignore all other events
                _ => (),
            }
        });
    });

    let output_stream = pa.open_non_blocking_stream(output_stream_settings, move |args| {
        let batch: Option<(StereoWaveform, Vec<f32>)> =
            render_manager.lock().unwrap().read(buffer_size);

        if let Some((b, ramp)) = batch {
            new_write_output_buffer(args.buffer, b.clone(), ramp.clone());

            // Send the audio buffer to the visualizer thread
            // if let Err(err) = tx.send(b) {
            // eprintln!("Failed to send audio buffer to visualizer thread: {}", err);
            // }

            pa::Continue
        } else {
            write_output_buffer(
                args.buffer,
                StereoWaveform::new(Settings::global().buffer_size),
            );

            pa::Continue
        }
    })?;

    Ok(output_stream)
}

struct VisualizerState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: winit::window::Window,
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

        VisualizerState {
            device,
            queue,
            window,
        }
    }

    fn visualize_rendered_buffer(&self, audio_buffer: Arc<StereoWaveform>) {
        let spectrum = self.to_spectrum(audio_buffer);

        let texture = self.create_texture_from_spectrum(&spectrum);

        self.draw_texture_to_window(&texture);
    }

    fn to_spectrum(&self, audio_buffer: Arc<StereoWaveform>) -> Vec<f32> {
        // TODO: Implement the details of the FFT transformation
        // Here is a simplified example:

        let mut planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_forward(audio_buffer.l_buffer.len());

        let mut complex = audio_buffer
            .l_buffer
            .iter()
            .map(|&x| rustfft::num_complex::Complex {
                re: x as f32,
                im: 0.0,
            })
            .collect::<Vec<_>>();

        fft.process(&mut complex);

        complex.iter().map(|x| x.norm()).collect::<Vec<f32>>()
    }

    fn create_texture_from_spectrum(&self, spectrum: &Vec<f32>) -> wgpu::Texture {
        let texture_size = wgpu::Extent3d {
            width: spectrum.len() as u32,
            height: 1,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Spectrum Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_data = spectrum
            .iter()
            .map(|&x| (x * 255.0) as u8)
            .collect::<Vec<_>>();

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(spectrum.len() as u32),
                rows_per_image: Some(1),
            },
            texture_size,
        );

        texture
    }

    fn create_render_pipeline(
        &self,
        swap_chain_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let vertex_shader_data =
            std::fs::read("shader.vert.spv").expect("Failed to read vertex shader");
        let vertex_shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Vertex Shader"),
                source: wgpu::util::make_spirv(&vertex_shader_data),
            });

        let fragment_shader_data =
            std::fs::read("shader.frag.spv").expect("Failed to read fragment shader");
        let fragment_shader_module =
            self.device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Fragment Shader"),
                    source: wgpu::util::make_spirv(&fragment_shader_data),
                });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                        format: swap_chain_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
            });

        render_pipeline
    }

    fn draw_texture_to_window(&self, texture: &wgpu::Texture) {
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let render_pipeline = self.create_render_pipeline(wgpu::TextureFormat::Bgra8UnormSrgb);
        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&render_pipeline);
            render_pass.draw(0..4, 0..1); // Draw a quad for the full screen
        }

        let command_buffer = command_encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));
    }
}

pub fn get_output_settings(pa: &pa::PortAudio) -> Result<pa::stream::OutputSettings<f32>, Error> {
    let def_output = pa.default_output_device()?;
    let output_info = pa.device_info(def_output)?;
    // println!("Default output device info: {:#?}", &output_info);
    let latency = output_info.default_low_output_latency;
    let output_params = pa::StreamParameters::new(
        def_output,
        Settings::global().channels,
        Settings::global().interleaved,
        latency,
    );

    let output_settings = pa::OutputStreamSettings::new(
        output_params,
        Settings::global().sample_rate,
        Settings::global().buffer_size as u32,
    );

    Ok(output_settings)
}
