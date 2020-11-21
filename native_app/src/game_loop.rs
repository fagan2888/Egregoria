use crate::audio::AudioHandle;
use crate::context::Context;
use crate::gui::windows::debug::DebugObjs;
use crate::gui::{setup_gui, FollowEntity, Gui, UiTextures};
use crate::rendering::imgui_wrapper::ImguiWrapper;
use crate::rendering::{CameraHandler, InstancedRender, MeshRenderer, RoadRenderer};
use common::GameTime;
use egregoria::engine_interaction::{KeyboardInfo, MouseInfo, RenderStats, TimeWarp};
use egregoria::rendering::immediate::{ImmediateDraw, ImmediateOrder, ImmediateSound, OrderKind};
use egregoria::souls::Souls;
use egregoria::{load_from_disk, Egregoria};
use geom::{vec3, Vec2};
use geom::{Camera, Vec3};
use map_model::Map;
use std::borrow::Cow;
use std::time::Instant;
use wgpu_engine::lighting::LightInstance;
use wgpu_engine::{FrameContext, GfxContext, GuiRenderContext};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct State {
    camera: CameraHandler,
    imgui_render: ImguiWrapper,
    goria: Egregoria,
    last_time: Instant,
    instanced_renderer: InstancedRender,
    road_renderer: RoadRenderer,
    gui: Gui,
    souls: Souls,
    music_h: AudioHandle,
}

impl State {
    pub fn new(ctx: &mut Context) -> Self {
        let camera = common::saveload::load("camera")
            .map(|camera| CameraHandler {
                camera,
                last_pos: Vec2::ZERO,
            })
            .unwrap_or_else(|| {
                CameraHandler::new(
                    ctx.gfx.size.0 as f32,
                    ctx.gfx.size.1 as f32,
                    vec3(0.0, 0.0, 20.0),
                )
            });

        let mut imgui_render = ImguiWrapper::new(&mut ctx.gfx, &ctx.window);

        crate::rendering::prepare_background(&mut ctx.gfx);

        let mut goria = egregoria::Egregoria::init();

        goria.insert(UiTextures::new(&ctx.gfx, &mut imgui_render.renderer));

        load_from_disk(&mut goria);
        setup_gui(&mut goria);

        let gui: Gui = common::saveload::load("gui").unwrap_or_default();

        goria.insert(camera.camera.clone());
        let music_h = ctx.audio.play_with_control("music1");

        Self {
            camera,
            imgui_render,
            goria,
            last_time: Instant::now(),
            instanced_renderer: InstancedRender::new(&mut ctx.gfx),
            road_renderer: RoadRenderer::new(&mut ctx.gfx),
            gui,
            souls: Souls::default(),
            music_h,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        let delta = self.last_time.elapsed().as_secs_f64();
        self.last_time = Instant::now();

        for sound in self.goria.write::<ImmediateSound>().orders.drain(..) {
            ctx.audio.play(sound)
        }

        self.manage_time(delta, &mut ctx.gfx);

        self.manage_io(ctx);

        self.camera.easy_camera_movement(
            ctx,
            delta as f32,
            !self.imgui_render.last_mouse_captured,
            !self.imgui_render.last_kb_captured,
        );
        *self.goria.write::<Camera>() = self.camera.camera.clone();

        if !self.imgui_render.last_mouse_captured {
            self.goria.write::<MouseInfo>().unprojected = self.unproject(ctx.input.mouse.screen);
        }

        self.goria.run();

        self.souls.add_souls_to_empty_buildings(&mut self.goria);
        self.souls.update(&mut self.goria);

        self.manage_entity_follow();
        self.camera.update(ctx);
    }

    pub fn render(&mut self, ctx: &mut FrameContext) {
        let start = Instant::now();

        crate::rendering::draw_background(ctx);

        let mut tess = self.camera.culled_tesselator();

        let time: GameTime = *self.goria.read::<GameTime>();
        self.road_renderer
            .render(&mut self.goria.write::<Map>(), time.seconds, &mut tess, ctx);

        self.instanced_renderer.render(&mut self.goria, ctx);

        MeshRenderer::render(&mut self.goria, &mut tess);

        {
            let objs = self.goria.read::<DebugObjs>();
            for (val, _, obj) in &objs.0 {
                if *val {
                    obj(&mut tess, &self.goria);
                }
            }
        }

        {
            let immediate = &mut *self.goria.write::<ImmediateDraw>();
            for ImmediateOrder { kind, color, z } in immediate
                .persistent_orders
                .iter()
                .chain(immediate.orders.iter())
            {
                let z = *z;
                tess.set_color(*color);
                match *kind {
                    OrderKind::Circle { pos, radius } => {
                        tess.draw_circle(pos, z, radius);
                    }
                    OrderKind::Line {
                        from,
                        to,
                        thickness,
                    } => {
                        tess.draw_stroke(from, to, z, thickness);
                    }
                    OrderKind::StrokeCircle {
                        pos,
                        radius,
                        thickness,
                    } => {
                        tess.draw_stroke_circle(pos, z, radius, thickness);
                    }
                    OrderKind::PolyLine {
                        ref points,
                        thickness,
                    } => {
                        tess.draw_polyline(points, z, thickness);
                    }
                    OrderKind::OBB(obb) => {
                        let [ax1, ax2] = obb.axis();
                        tess.draw_rect_cos_sin(
                            obb.center(),
                            z,
                            ax1.magnitude(),
                            ax2.magnitude(),
                            ax1.normalize(),
                        );
                    }
                }
            }
            immediate.orders.clear();
        }

        if let Some(x) = tess.meshbuilder.build(ctx.gfx) {
            ctx.draw(x)
        }

        self.goria
            .write::<RenderStats>()
            .render
            .add_value(start.elapsed().as_secs_f32());
    }

    pub fn lights(&self) -> (Cow<[LightInstance]>, Vec3) {
        let mut lights = vec![];

        let time = self.goria.read::<GameTime>();
        let daysec = time.daysec();

        let map = self.goria.read::<Map>();
        for x in map.roads().values() {
            let w = x.width * 0.5 - 5.0;
            for (point, dir) in x.generated_points().equipoints_dir(45.0) {
                lights.push(LightInstance {
                    pos: (point + dir.perpendicular() * w).into(),
                    scale: 60.0,
                });
                lights.push(LightInstance {
                    pos: (point - dir.perpendicular() * w).into(),
                    scale: 60.0,
                });
            }
        }

        for i in map.intersections().values() {
            lights.push(LightInstance {
                pos: (i.pos).into(),
                scale: 60.0,
            });
        }

        let dark = vec3(0.1, 0.1, 0.1);
        let bright = vec3(1.0, 1.0, 1.0);

        let col = match time.daytime.hour {
            0..=5 => dark,
            6..=9 => {
                let c = (daysec / GameTime::HOUR as f64 - 6.0) / 4.0;
                dark.smoothstep(bright, c as f32)
            }
            10..=15 => bright,
            16..=20 => {
                let c = (daysec / GameTime::HOUR as f64 - 16.0) / 5.0;
                bright.smoothstep(dark, c as f32)
            }
            21..=24 => dark,
            _ => dark,
        };

        (Cow::Owned(lights), col)
    }

    pub fn render_gui(&mut self, window: &Window, ctx: GuiRenderContext) {
        self.imgui_render
            .render(ctx, window, &mut self.goria, &mut self.gui);
    }

    fn manage_time(&mut self, delta: f64, gfx: &mut GfxContext) {
        const MAX_TIMESTEP: f64 = 1.0 / 15.0;

        let mut time = self.goria.write::<GameTime>();
        let warp = self.goria.read::<TimeWarp>().0;

        let delta = (delta * warp as f64).min(MAX_TIMESTEP);

        *time = GameTime::new(delta as f32, time.timestamp + delta);

        gfx.set_time(time.timestamp as f32);
    }

    fn manage_entity_follow(&mut self) {
        if !self.goria.read::<MouseInfo>().just_pressed.is_empty() {
            self.goria.write::<FollowEntity>().0.take();
        }

        if let Some(e) = self.goria.read::<FollowEntity>().0 {
            if let Some(pos) = self.goria.pos(e) {
                self.camera.camera.position.x = pos.x;
                self.camera.camera.position.y = pos.y;
            }
        }
    }

    fn manage_io(&mut self, ctx: &Context) {
        *self.goria.write::<KeyboardInfo>() = ctx.input.keyboard.clone();
        *self.goria.write::<MouseInfo>() = ctx.input.mouse.clone();

        if self.imgui_render.last_kb_captured {
            let kb: &mut KeyboardInfo = &mut self.goria.write::<KeyboardInfo>();
            kb.just_pressed.clear();
            kb.is_pressed.clear();
        }

        if self.imgui_render.last_mouse_captured {
            let mouse: &mut MouseInfo = &mut self.goria.write::<MouseInfo>();
            mouse.just_pressed.clear();
            mouse.buttons.clear();
            mouse.wheel_delta = 0.0;
        }
    }

    pub fn event(&mut self, window: &Window, event: &winit::event::Event<()>) {
        self.imgui_render.handle_event(window, event);
    }

    pub fn resized(&mut self, ctx: &mut Context, size: PhysicalSize<u32>) {
        self.camera
            .resize(ctx, size.width as f32, size.height as f32);
    }

    pub fn unproject(&self, pos: Vec2) -> Vec2 {
        self.camera.unproject_mouse_click(pos)
    }
}
