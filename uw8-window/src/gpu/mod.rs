use crate::Framebuffer;
use anyhow::{anyhow, Result};
use std::{num::NonZeroU32, time::Instant};

use winit::{
    dpi::PhysicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

#[cfg(target_os = "macos")]
use winit::platform::macos::EventLoopExtMacOS;
#[cfg(target_os = "linux")]
use winit::platform::unix::EventLoopExtUnix;
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopExtWindows;

mod crt;
mod fast_crt;
mod square;

use crt::CrtFilter;
use fast_crt::FastCrtFilter;
use square::SquareFilter;

pub struct Window {
    event_loop: EventLoop<()>,
    window: winit::window::Window,
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Window {
    pub fn new() -> Result<Window> {
        async fn create() -> Result<Window> {
            let event_loop = EventLoop::new_any_thread();
            let window = WindowBuilder::new()
                .with_inner_size(PhysicalSize::new(640u32, 480))
                .with_min_inner_size(PhysicalSize::new(320u32, 240))
                .with_title("MicroW8")
                .build(&event_loop)?;

            window.set_cursor_visible(false);

            let instance = wgpu::Instance::new(wgpu::Backends::all());
            let surface = unsafe { instance.create_surface(&window) };
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .ok_or_else(|| anyhow!("Request adapter failed"))?;

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor::default(), None)
                .await?;

            Ok(Window {
                event_loop,
                window,
                instance,
                surface,
                adapter,
                device,
                queue,
            })
        }

        pollster::block_on(create())
    }

    pub fn run(
        self,
        mut update: Box<dyn FnMut(&mut dyn Framebuffer, u32, bool) -> Instant + 'static>,
    ) -> ! {
        let Window {
            event_loop,
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
        } = self;

        let palette_screen_mode = PaletteScreenMode::new(&device);

        let mut surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
        };

        let mut filter: Box<dyn Filter> = Box::new(CrtFilter::new(
            &device,
            &palette_screen_mode.screen_view,
            window.inner_size(),
            surface_config.format,
        ));

        surface.configure(&device, &surface_config);

        let mut reset = false;
        let mut gamepad = 0;

        event_loop.run(move |event, _, control_flow| {
            let _ = (&window, &instance, &surface, &adapter, &device);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(new_size) => {
                        surface_config.width = new_size.width;
                        surface_config.height = new_size.height;
                        surface.configure(&device, &surface_config);
                        filter.resize(&queue, new_size);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        fn gamepad_button(input: &winit::event::KeyboardInput) -> u32 {
                            match input.scancode {
                                44 => 16,
                                45 => 32,
                                30 => 64,
                                31 => 128,
                                _ => match input.virtual_keycode {
                                    Some(VirtualKeyCode::Up) => 1,
                                    Some(VirtualKeyCode::Down) => 2,
                                    Some(VirtualKeyCode::Left) => 4,
                                    Some(VirtualKeyCode::Right) => 8,
                                    _ => 0,
                                },
                            }
                        }
                        if input.state == winit::event::ElementState::Pressed {
                            match input.virtual_keycode {
                                Some(VirtualKeyCode::Escape) => *control_flow = ControlFlow::Exit,
                                Some(VirtualKeyCode::F) => {
                                    window.set_fullscreen(if window.fullscreen().is_some() {
                                        None
                                    } else {
                                        Some(Fullscreen::Borderless(None))
                                    });
                                }
                                Some(VirtualKeyCode::R) => reset = true,
                                Some(VirtualKeyCode::Key1) => {
                                    filter = Box::new(SquareFilter::new(
                                        &device,
                                        &palette_screen_mode.screen_view,
                                        window.inner_size(),
                                        surface_config.format,
                                    ))
                                }
                                Some(VirtualKeyCode::Key2) => {
                                    filter = Box::new(FastCrtFilter::new(
                                        &device,
                                        &palette_screen_mode.screen_view,
                                        window.inner_size(),
                                        surface_config.format,
                                    ))
                                }
                                Some(VirtualKeyCode::Key3) => {
                                    filter = Box::new(CrtFilter::new(
                                        &device,
                                        &palette_screen_mode.screen_view,
                                        window.inner_size(),
                                        surface_config.format,
                                    ))
                                }
                                _ => (),
                            }

                            gamepad |= gamepad_button(&input);
                        } else {
                            gamepad &= !gamepad_button(&input);
                        }
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    if let ControlFlow::WaitUntil(t) = *control_flow {
                        if Instant::now() < t {
                            return;
                        }
                    }
                    let next_frame = update(
                        &mut GpuFramebuffer {
                            queue: &queue,
                            framebuffer: &palette_screen_mode,
                        },
                        gamepad,
                        reset,
                    );
                    reset = false;
                    *control_flow = ControlFlow::WaitUntil(next_frame);

                    let output = surface.get_current_texture().unwrap();
                    let view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    palette_screen_mode.resolve_screen(&mut encoder);

                    {
                        let mut render_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 0.0,
                                            g: 0.0,
                                            b: 0.0,
                                            a: 1.0,
                                        }),
                                        store: true,
                                    },
                                })],
                                depth_stencil_attachment: None,
                            });

                        filter.render(&mut render_pass);
                    }

                    queue.submit(std::iter::once(encoder.finish()));
                    output.present();
                }
                _ => (),
            }
        });
    }
}

struct GpuFramebuffer<'a> {
    framebuffer: &'a PaletteScreenMode,
    queue: &'a wgpu::Queue,
}

impl<'a> Framebuffer for GpuFramebuffer<'a> {
    fn update(&mut self, pixels: &[u8], palette: &[u8]) {
        self.framebuffer.write_framebuffer(self.queue, pixels);
        self.framebuffer.write_palette(self.queue, palette);
    }
}

trait Filter {
    fn resize(&self, queue: &wgpu::Queue, new_size: PhysicalSize<u32>);
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}

struct PaletteScreenMode {
    framebuffer: wgpu::Texture,
    palette: wgpu::Texture,
    screen_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl PaletteScreenMode {
    fn new(device: &wgpu::Device) -> PaletteScreenMode {
        let framebuffer_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 320,
                height: 240,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
        });

        let palette_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
        });

        let screen_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 320,
                height: 240,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        });

        let framebuffer_texture_view =
            framebuffer_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let palette_texture_view =
            palette_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let screen_texture_view =
            screen_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let palette_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D1,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let palette_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &palette_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&framebuffer_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&palette_texture_view),
                },
            ],
            label: None,
        });

        let palette_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("palette.wgsl").into()),
        });

        let palette_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&palette_bind_group_layout],
                push_constant_ranges: &[],
            });

        let palette_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&palette_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &palette_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &palette_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });

        PaletteScreenMode {
            framebuffer: framebuffer_texture,
            palette: palette_texture,
            screen_view: screen_texture_view,
            bind_group: palette_bind_group,
            pipeline: palette_pipeline,
        }
    }

    fn write_framebuffer(&self, queue: &wgpu::Queue, pixels: &[u8]) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.framebuffer,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytemuck::cast_slice(pixels),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(320),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 320,
                height: 240,
                depth_or_array_layers: 1,
            },
        );
    }

    fn write_palette(&self, queue: &wgpu::Queue, palette: &[u8]) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.palette,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytemuck::cast_slice(palette),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(256 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }

    fn resolve_screen(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.screen_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}