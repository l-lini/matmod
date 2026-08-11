use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{
        ActiveEventLoop,
        EventLoop,
    },
    window::{
        Window,
        WindowId,
    },
};
use wgpu::{
    Color,
    Queue,
    Device,
    LoadOp,
    StoreOp,
    Surface,
    Backends,
    Features,
    Instance,
    Operations,
    PresentMode,
    TextureUsages,
    TextureFormat,
    DeviceDescriptor,
    SurfaceColorSpace,
    CompositeAlphaMode,
    InstanceDescriptor,
    SurfaceConfiguration,
    RenderPassDescriptor,
    TextureViewDescriptor,
    RequestAdapterOptions,
    CommandEncoderDescriptor,
    RenderPassColorAttachment,
    CurrentSurfaceTexture::Success,
};
use tokio::runtime::Runtime;
use std::sync::Arc;

#[derive(Default)]
struct Game<'s> {
    window: Option<Arc<Window>>,
    surface: Option<Surface<'s>>,
    device: Option<Device>,
    queue: Option<Queue>,
}

impl<'s> ApplicationHandler for Game<'s> {
    fn resumed(&mut self, event_loop: & ActiveEventLoop) {
        self.window = Some(
            Arc::new(
                event_loop.create_window(
                    Window::default_attributes()
                ).unwrap()
            )
        );

        let display = event_loop.owned_display_handle();

        let instance = Instance::new(
            InstanceDescriptor::new_with_display_handle(Box::new(display))
        );

        self.surface = instance.create_surface(
            self
            .window
            .as_ref()
            .unwrap()
            .clone()
        )
        .ok();

        let rt = Runtime::new().unwrap();
        let adapter = rt.block_on(
            instance.request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(self.surface.as_ref().unwrap()),
                ..Default::default()
            })
        ).unwrap();
        let adapters = rt.block_on(
            instance.enumerate_adapters(
                Backends::all()
            )
        );
        dbg!(adapters.into_iter().map(|a| a.get_info()).collect::<Vec<_>>());

        let (device, queue) = rt.block_on(
            adapter.request_device(
                &DeviceDescriptor {
                    required_features: Features::TEXTURE_FORMAT_16BIT_NORM,
                    ..Default::default()
                }
            )
        ).unwrap();
        self.queue = Some(queue);
        self.device = Some(device);

        self.surface.as_ref().unwrap().configure(self.device.as_ref().unwrap(), &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Rgba8UnormSrgb,
            color_space: SurfaceColorSpace::Auto,
            width: self.window.as_ref().unwrap().inner_size().width,
            height: self.window.as_ref().unwrap().inner_size().height,
            present_mode: PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let (Some(surface), Some(queue), Some(device)) = (self.surface.as_ref(), self.queue.as_ref(), self.device.as_ref()) {
                    if let Success(texture) = (&surface).get_current_texture() {
                        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
                        {
                            let view = texture.texture.create_view(&TextureViewDescriptor {
                                format: Some(TextureFormat::Rgba8UnormSrgb),
                                ..Default::default()
                            });
                            let _render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                                color_attachments: &[
                                    Some(RenderPassColorAttachment {
                                        view: &view,
                                        depth_slice: None,
                                        resolve_target: None,
                                        ops: Operations {
                                            load: LoadOp::Clear(Color::WHITE),
                                            store: StoreOp::Store,
                                        },
                                    }),
                                ],
                                ..Default::default()
                            });
                        }
                        let command_buffers = vec![command_encoder.finish()];
                        queue.submit(command_buffers);
                        queue.present(texture);
                    }
                }
                self.window.as_ref().unwrap().request_redraw();
            },
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut game = Game::default();
    let _ = event_loop.run_app(&mut game);
}
