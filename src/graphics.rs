use bytemuck::NoUninit;
use cgmath::{Matrix4, Point3, Rad, Vector3, perspective, prelude::*};
use std::{borrow::Cow, sync::Arc};
use tokio::runtime::Runtime;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor,
    BufferUsages, Color, CommandEncoderDescriptor, CompareFunction, CurrentSurfaceTexture::*,
    DepthBiasState, DepthStencilState, Device, DeviceDescriptor, Extent3d, FragmentState, Instance,
    InstanceDescriptor, LoadOp, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PresentMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StencilState, StoreOp, Surface, SurfaceConfiguration,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
    VertexBufferLayout, VertexState, VertexStepMode, util::BufferInitDescriptor, util::DeviceExt,
    vertex_attr_array,
};
use winit::{event_loop::ActiveEventLoop, window::Window};

const MAX_BALLS: u64 = 10_000;

const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

#[repr(C, packed)]
#[derive(NoUninit, Copy, Clone, Debug)]
pub struct BoxVertex {
    pub position: [f32; 4],
}

#[repr(C, packed)]
#[derive(NoUninit, Copy, Clone, Debug)]
pub struct BallVertex {
    pub position: [f32; 4],
    pub texture_position: [f32; 2],
}

pub struct Graphics<'surface> {
    queue: Queue,
    device: Device,
    window: Arc<Window>,
    box_vertex_buffer: Buffer,
    ball_vertex_buffer: Buffer,
    camera_buffer: Buffer,
    bind_group: BindGroup,
    view_matrix: Matrix4<f32>,
    projection_matrix: Matrix4<f32>,
    box_verticies: u32,
    ball_verticies: u32,
    surface: Surface<'surface>,
    near: f32,
    far: f32,
    fovy: Rad<f32>,
    box_pipeline: RenderPipeline,
    ball_pipeline: RenderPipeline,
    surface_configuration: SurfaceConfiguration,
}

impl<'surface> Graphics<'surface> {
    pub fn new(
        event_loop: &ActiveEventLoop,
        view_matrix: Matrix4<f32>,
        fovy: Rad<f32>,
        near: f32,
        far: f32,
        box_verticies: &[BoxVertex],
    ) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let display = event_loop.owned_display_handle();

        let instance = Instance::new(InstanceDescriptor::new_with_display_handle(Box::new(
            display,
        )));

        let surface = instance.create_surface(window.clone()).unwrap();

        let rt = Runtime::new().unwrap();

        let adapter = rt
            .block_on(instance.request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }))
            .unwrap();

        let (device, queue) = rt
            .block_on(adapter.request_device(&DeviceDescriptor::default()))
            .unwrap();

        let [width, height] = window.inner_size().into();

        let mut surface_configuration =
            surface.get_default_config(&adapter, width, height).unwrap();
        surface_configuration.present_mode = PresentMode::Fifo;

        surface.configure(&device, &surface_configuration);

        let wireframe_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("wireframe_shader.wgsl"))),
        });

        let ball_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("ball_shader.wgsl"))),
        });

        let box_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex buffer"),
            usage: BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(box_verticies),
        });

        let projection_matrix = perspective(fovy, width as f32 / height as f32, near, far);
        let vp_mat = OPENGL_TO_WGPU_MATRIX * projection_matrix * view_matrix;
        let vp_ref: &[f32; 16] = vp_mat.as_ref();

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(vp_ref),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Bind group"),
        });

        let box_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let box_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&box_pipeline_layout),
            vertex: VertexState {
                module: &wireframe_shader,
                entry_point: Some("vs_main"),
                buffers: &vec![Some(VertexBufferLayout {
                    array_stride: size_of::<BoxVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x4,
                    ],
                })],
                compilation_options: PipelineCompilationOptions {
                    constants: &vec![],
                    zero_initialize_workgroup_memory: true,
                },
            },
            fragment: Some(FragmentState {
                module: &wireframe_shader,
                entry_point: Some("fs_main"),
                targets: &vec![Some(TextureFormat::Bgra8UnormSrgb.into())],
                compilation_options: PipelineCompilationOptions {
                    constants: &vec![],
                    zero_initialize_workgroup_memory: true,
                },
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                ..Default::default()
            },
            multisample: MultisampleState::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::LessEqual),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let ball_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let ball_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&ball_pipeline_layout),
            vertex: VertexState {
                module: &ball_shader,
                entry_point: Some("vs_main"),
                buffers: &vec![Some(VertexBufferLayout {
                    array_stride: size_of::<BallVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x4,
                        1 => Float32x2,
                    ],
                })],
                compilation_options: PipelineCompilationOptions {
                    constants: &vec![],
                    zero_initialize_workgroup_memory: true,
                },
            },
            fragment: Some(FragmentState {
                module: &ball_shader,
                entry_point: Some("fs_main"),
                targets: &vec![Some(TextureFormat::Bgra8UnormSrgb.into())],
                compilation_options: PipelineCompilationOptions {
                    constants: &vec![],
                    zero_initialize_workgroup_memory: true,
                },
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                ..Default::default()
            },
            multisample: MultisampleState::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::LessEqual),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let ball_verticies = 0;
        let ball_vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: MAX_BALLS * 6 * size_of::<BallVertex>() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            near,
            far,
            fovy,
            box_verticies: box_verticies.len() as u32,
            ball_verticies,
            ball_vertex_buffer,
            bind_group,
            queue,
            device,
            window,
            surface,
            box_vertex_buffer,
            camera_buffer,
            view_matrix,
            projection_matrix,
            // instance,
            // shader,
            box_pipeline,
            ball_pipeline,
            surface_configuration,
        }
    }

    pub fn set_ball_verticies(&mut self, verticies: &[BallVertex]) {
        self.ball_verticies = verticies.len() as u32;
        self.queue
            .write_buffer(&self.ball_vertex_buffer, 0, bytemuck::cast_slice(verticies));
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_configuration.width = width;
        self.surface_configuration.height = height;
        self.surface
            .configure(&self.device, &self.surface_configuration);
        self.projection_matrix =
            perspective(self.fovy, width as f32 / height as f32, self.near, self.far);
        let vp_mat = OPENGL_TO_WGPU_MATRIX * self.projection_matrix * self.view_matrix;
        let vp_ref: &[f32; 16] = vp_mat.as_ref();
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(vp_ref));
    }

    pub fn draw(&self) {
        match self.surface.get_current_texture() {
            Success(texture) => {
                let mut command_encoder = self
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor::default());
                {
                    let view = &texture.texture.create_view(&TextureViewDescriptor {
                        format: Some(self.surface_configuration.format),
                        ..Default::default()
                    });

                    let depth_texture = self.device.create_texture(&TextureDescriptor {
                        size: Extent3d {
                            width: self.surface_configuration.width,
                            height: self.surface_configuration.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: TextureDimension::D2,
                        format: TextureFormat::Depth32Float,
                        view_formats: &[],
                        usage: TextureUsages::RENDER_ATTACHMENT,
                        label: None,
                    });

                    let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

                    let mut render_pass =
                        command_encoder.begin_render_pass(&RenderPassDescriptor {
                            color_attachments: &vec![Some(RenderPassColorAttachment {
                                view: &view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: Operations {
                                    load: LoadOp::Clear(Color::BLACK),
                                    store: StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                                view: &depth_view,
                                depth_ops: Some(Operations {
                                    load: LoadOp::Clear(1.0),
                                    store: StoreOp::Discard,
                                }),
                                stencil_ops: None,
                            }),
                            ..Default::default()
                        });

                    render_pass.set_pipeline(&self.box_pipeline);
                    render_pass.set_vertex_buffer(0, self.box_vertex_buffer.slice(..));
                    render_pass.set_bind_group(0, &self.bind_group, &[]);
                    render_pass.draw(0..self.box_verticies, 0..1);

                    render_pass.set_pipeline(&self.ball_pipeline);
                    render_pass.set_vertex_buffer(0, self.ball_vertex_buffer.slice(..));
                    render_pass.set_bind_group(0, &self.bind_group, &[]);
                    render_pass.draw(0..self.ball_verticies, 0..1);
                }

                let command_buffers = vec![command_encoder.finish()];
                self.queue.submit(command_buffers);

                self.queue.present(texture);

                self.window.request_redraw();
            }
            e => todo!("{:?}", e),
        }
    }
}
