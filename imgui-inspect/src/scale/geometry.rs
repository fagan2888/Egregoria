use crate::default::InspectArgsDefault;
use crate::default::InspectRenderDefault;
use geom::{PolyLine, Transform, Vec2};
use imgui::{im_str, Ui};

impl InspectRenderDefault<Transform> for Transform {
    fn render(data: &[&Transform], label: &'static str, ui: &Ui, args: &InspectArgsDefault) {
        let mut t = *data[0];
        Self::render_mut(&mut [&mut t], label, ui, args);
    }

    fn render_mut(
        data: &mut [&mut Transform],
        _: &'static str,
        ui: &Ui,
        _: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = &mut data[0];
        let mut position = x.position();
        let mut direction = x.direction();
        let mut changed = <Vec2 as InspectRenderDefault<Vec2>>::render_mut(
            &mut [&mut position],
            "position",
            ui,
            &InspectArgsDefault::default(),
        );
        changed |= <InspectVec2Rotation as InspectRenderDefault<Vec2>>::render_mut(
            &mut [&mut direction],
            "direction",
            ui,
            &InspectArgsDefault::default(),
        );
        x.set_direction(direction);
        x.set_position(position);
        changed
    }
}

pub struct InspectVec2Immutable;
impl InspectRenderDefault<Vec2> for InspectVec2Immutable {
    fn render(data: &[&Vec2], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = data[0];
        imgui::InputFloat2::new(ui, &im_str!("{}", label), &mut [x.x, x.y])
            .always_insert_mode(false)
            .build();
    }

    fn render_mut(
        data: &mut [&mut Vec2],
        label: &'static str,

        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        Self::render(&[&*data[0]], label, ui, args);
        false
    }
}

impl InspectRenderDefault<Vec2> for Vec2 {
    fn render(data: &[&Vec2], label: &'static str, ui: &imgui::Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = data[0];
        imgui::InputFloat2::new(ui, &im_str!("{}", label), &mut [x.x, x.y])
            .always_insert_mode(false)
            .build();
    }

    fn render_mut(
        data: &mut [&mut Vec2],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = &mut data[0];
        let mut conv = [x.x, x.y];
        let changed = ui
            .drag_float2(&im_str!("{}", label), &mut conv)
            .speed(args.step.unwrap_or(0.1))
            .build();
        x.x = conv[0];
        x.y = conv[1];
        changed
    }
}

impl InspectRenderDefault<PolyLine> for PolyLine {
    fn render(
        _data: &[&PolyLine],
        _label: &'static str,
        _ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut PolyLine],
        label: &str,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = &mut data[0];
        let mut changed = false;

        if imgui::CollapsingHeader::new(&im_str!("{}", label)).build(&ui) {
            ui.indent();
            for (i, x) in v.iter_mut().enumerate() {
                let id = ui.push_id(i as i32);
                changed |= <Vec2 as InspectRenderDefault<Vec2>>::render_mut(&mut [x], "", ui, args);
                id.pop(ui);
            }
            ui.unindent();
        }

        changed
    }
}

pub struct InspectVec2Rotation;
impl InspectRenderDefault<Vec2> for InspectVec2Rotation {
    fn render(data: &[&Vec2], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = data[0];
        let ang = x.angle(Vec2::UNIT_Y);
        ui.text(&im_str!("{} {}", label, ang));
    }

    fn render_mut(
        data: &mut [&mut Vec2],
        label: &'static str,

        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = &mut data[0];
        let mut ang = f32::atan2(x.y, x.x);

        let changed = ui
            .drag_float(&im_str!("{}", label), &mut ang)
            .speed(-args.step.unwrap_or(0.1))
            .build();
        x.x = ang.cos();
        x.y = ang.sin();
        changed
    }
}
