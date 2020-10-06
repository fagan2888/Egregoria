use crate::audio::AudioContext;
use crate::game_loop;
use crate::input::InputContext;
use futures::executor;
use geom::Vec3;
use wgpu_engine::GfxContext;
use winit::window::Window;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct Context {
    pub gfx: GfxContext,
    pub input: InputContext,
    pub audio: AudioContext,
    pub window: Window,
    pub el: Option<EventLoop<()>>,
}

impl Context {
    pub fn new() -> Self {
        let el = EventLoop::new();

        let size = el.primary_monitor().size();

        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(
                size.width as f32 * 0.8,
                size.height as f32 * 0.8,
            ))
            .with_title("Egregoria 0.1")
            .build(&el)
            .expect("Failed to create window");

        let gfx = executor::block_on(GfxContext::new(
            &window,
            window.inner_size().width,
            window.inner_size().height,
        ));
        let input = InputContext::default();
        let audio = AudioContext::new();

        Self {
            gfx,
            input,
            audio,
            window,
            el: Some(el),
        }
    }

    pub fn start(mut self, mut state: game_loop::State) {
        let mut frame: Option<_> = None;
        let mut new_size: Option<PhysicalSize<u32>> = None;

        self.el.take().unwrap().run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            state.event(&self.window, &event);
            match event {
                Event::WindowEvent { event, .. } => {
                    let managed = self.input.handle(&event);

                    if !managed {
                        match event {
                            WindowEvent::Resized(physical_size) => {
                                new_size = Some(physical_size);
                            }
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            _ => (),
                        }
                    }
                }
                Event::MainEventsCleared => match frame.take() {
                    None => {
                        if let Some(new_size) = new_size.take() {
                            self.gfx.resize(new_size.width, new_size.height);
                            state.resized(&mut self, new_size);
                        }
                        frame = Some(
                            self.gfx
                                .swapchain
                                .get_current_frame()
                                .expect("Error getting swapchain frame"),
                        );
                    }
                    Some(sco) => {
                        self.input.mouse.unprojected = state.unproject(self.input.mouse.screen);

                        state.update(&mut self);

                        let window = &self.window;
                        let mut enc = self.gfx.start_frame();

                        self.gfx.render_objs(&mut enc, &sco, |fc| state.render(fc));

                        wgpu_engine::lighting::render_lights(
                            &self.gfx,
                            &mut enc,
                            &sco,
                            &state.lights(),
                            Vec3::new(0.7, 0.7, 1.0),
                        );

                        self.gfx
                            .render_gui(&mut enc, &sco, |gctx| state.render_gui(window, gctx));

                        self.gfx.finish_frame(enc);

                        self.input.end_frame();
                    }
                },
                _ => (),
            }
        })
    }
}
