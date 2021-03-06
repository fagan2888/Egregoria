use crate::audio::{AudioContext, AudioHandle};
use common::AudioKind;
use egregoria::physics::CollisionWorld;
use egregoria::Egregoria;
use flat_spatial::grid::GridHandle;
use geom::Camera;
use rodio::Source;
use slotmap::SecondaryMap;
use std::time::Duration;

pub struct CarSounds {
    pub speed: f32,
    pub sounds: SecondaryMap<GridHandle, (AudioHandle, f32)>,
}

impl CarSounds {
    pub fn new(_ctx: &mut AudioContext) -> Self {
        Self {
            speed: 0.0,
            sounds: SecondaryMap::new(),
        }
    }

    pub fn update(&mut self, goria: &Egregoria, ctx: &mut AudioContext, _delta: f32) {
        let coworld = goria.read::<CollisionWorld>();
        let cam = goria.read::<Camera>();
        let campos = cam.position;
        let cambbox = cam.get_screen_box().expand(30.0);

        // Check too high
        if campos.z > 150.0 {
            for (_, (s, _)) in self.sounds.drain() {
                ctx.stop(s);
            }

            return;
        }

        // Remove sounds outside screen
        let mut to_remove = vec![];

        for (h, _) in &self.sounds {
            if let Some((pos, _)) = coworld.get(h) {
                if cambbox.contains(pos) {
                    continue;
                }
            }
            to_remove.push(h);
        }

        for h in to_remove {
            let (a_h, _) = self.sounds.remove(h).unwrap();
            ctx.stop(a_h);
        }

        // Gather
        for (h, _) in coworld.query_aabb(cambbox.ll, cambbox.ur) {
            let (pos, obj) = coworld.get(h).unwrap();
            if !matches!(obj.group, egregoria::physics::PhysicsGroup::Vehicles) {
                continue;
            }

            if self.sounds.len() >= 30 {
                break;
            }

            if !self.sounds.contains_key(h) {
                let a = ctx.play_with_control(
                    "car_loop",
                    |x| {
                        x.repeat_infinite().skip_duration(Duration::from_micros(
                            (common::rand::rand2(pos.x, pos.y) * 10000.0) as u64,
                        ))
                    },
                    AudioKind::Effect,
                    true,
                );
                self.sounds.insert(h, (a, obj.speed));
            }
        }

        // Update
        for (h, (a_h, prev_speed)) in &mut self.sounds {
            let (pos, obj) = coworld.get(h).unwrap(); // Unwrap ok: checked it existed before

            ctx.set_volume(*a_h, 1.0 / (1.0 + 0.01 * pos.z(0.0).distance2(campos)));

            let mut acc = 0.0;
            if obj.speed > *prev_speed {
                acc = 0.5 / (1.0 + 0.3 * obj.speed);
            }
            ctx.set_speed(*a_h, 0.4 + obj.speed / 40.0 + acc);

            *prev_speed = obj.speed;
        }
    }
}
