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
    Instance,
    Operations,
    TextureFormat,
    DeviceDescriptor,
    InstanceDescriptor,
    SurfaceConfiguration,
    RenderPassDescriptor,
    TextureViewDescriptor,
    RequestAdapterOptions,
    CommandEncoderDescriptor,
    RenderPassColorAttachment,
    CurrentSurfaceTexture::*,
};
use tokio::runtime::Runtime;
use std::sync::Arc;

enum Game<'s> {
    Uninitialized,
    Initialized {
        queue: Queue,
        device: Device,
        window: Arc<Window>,
        surface: Surface<'s>,
        instance: Instance,
        surface_configuration: SurfaceConfiguration,
    }
}

impl<'s> ApplicationHandler for Game<'s> {
    fn resumed(&mut self, event_loop: & ActiveEventLoop) {
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
            ).unwrap()
        );

        let display = event_loop.owned_display_handle();

        let instance = Instance::new(
            InstanceDescriptor::new_with_display_handle(
                Box::new(display)
            )
        );

        let surface = instance.create_surface(window.clone()).unwrap();

        let rt = Runtime::new().unwrap();

        let adapter = rt.block_on(
            instance.request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
        ).unwrap();

        let (device, queue) = rt.block_on(
            adapter.request_device(
                &DeviceDescriptor::default()
            )
        ).unwrap();

        let [width, height] = window.inner_size().into();

        let surface_configuration = surface.get_default_config(
            &adapter,
            width,
            height,
        ).unwrap();

        surface.configure(&device, &surface_configuration);

        *self = Game::Initialized {
            queue,
            device,
            window,
            surface,
            instance,
            surface_configuration,
        };
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
                if let Game::Initialized { surface, queue, device, window, surface_configuration, instance, .. } = self {
                    match surface.get_current_texture() {
                        Success(texture) => {
                            let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
                            {
                                let view = &texture.texture.create_view(&TextureViewDescriptor {
                                    format: Some(TextureFormat::Bgra8UnormSrgb),
                                    ..Default::default()
                                });

                                let _ = command_encoder.begin_render_pass(&RenderPassDescriptor {
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

                            // window.pre_present_notify();
                            queue.present(texture);

                            window.request_redraw()
                        },
                        Suboptimal(_) => todo!("highly recomended to reconfigure"),
                        Timeout => todo!("skip and try again later"),
                        Occluded => todo!("skip frame and wait until not occluded"),
                        Outdated => todo!("configure and try again"),
                        Lost => todo!("check if device lost, try to recreate surface"),
                        Validation => todo!("attend to the validation error"),
                    }
                }
            },
            _ => { dbg!(event); },
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut game = Game::Uninitialized;
    let _ = event_loop.run_app(&mut game);
}
