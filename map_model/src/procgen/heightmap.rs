use geom::{vec2, vec3, Vec2, Vec3};

fn permute(x: f32) -> f32 {
    ((x * 34.0 + 1.0) * x) % 289.0
}

const CX: Vec2 = Vec2::new(0.211_324_87, 0.211_324_87);
const CY: Vec2 = Vec2::new(0.366_025_42, 0.366_025_42);
const CZ: Vec2 = Vec2::new(-0.577_350_26, -0.577_350_26);
const K: f32 = 0.024_390_243;

// Gradient mapping with an extra rotation.
pub fn grad2(p: Vec2) -> Vec2 {
    // Map from a line to a diamond such that a shift maps to a rotation.
    let u = permute(permute(p.x) + p.y) * K;
    let u = 4.0 * u.fract() - 2.0;
    vec2(u.abs() - 1.0, ((u + 1.0).abs() - 2.0).abs() - 1.0)
}

#[allow(clippy::many_single_char_names)]
pub fn simplex_noise(pos: Vec2) -> f32 {
    let mut i: Vec2 = Vec2::floor(pos + Vec2::splat(Vec2::dot(pos, CY)));
    let x0: Vec2 = pos - i + Vec2::splat(Vec2::dot(i, CX));
    let i1 = if x0.x > x0.y {
        vec2(1.0, 0.0)
    } else {
        vec2(0.0, 1.0)
    };
    let v1 = x0 + CX - i1;
    let v2 = x0 + CZ;

    i.x %= 289.0;
    i.y %= 289.0;

    let t: Vec3 = (Vec3::splat(0.5) - vec3(x0.magnitude2(), v1.magnitude2(), v2.magnitude2()))
        .max(Vec3::ZERO);
    let t2: Vec3 = t * t;
    let t4 = t2 * t2;

    let g0 = grad2(i);
    let g1 = grad2(i + i1);
    let g2 = grad2(i + Vec2::splat(1.0));

    let gv = vec3(g0.dot(x0), g1.dot(v1), g2.dot(v2));

    /*
        // Compute partial derivatives in x and y
        let temp = t2 * t * gv;
        let mut grad = -8.0
            * vec2(
                temp.dot(vec3(x0.x, v1.x, v2.x)),
                temp.dot(vec3(x0.y, v1.y, v2.y)),
            );
        grad.x += t4.dot(vec3(g0.x, g1.x, g2.x));
        grad.y += t4.dot(vec3(g0.y, g1.y, g2.y));
        grad = 40.0 * grad;
    */

    40.0 * t4.dot(gv)
}

const FBM_MAG: f32 = 0.4;

fn fnoise(ampl: f32, in_wv: Vec2) -> f32 {
    let mut dec = Vec2::splat(70.69) + in_wv * ampl;

    let mut noise: f32 = 0.0;
    let mut amplitude: f32 = 0.5;

    for _ in 0..8 {
        noise += amplitude * (simplex_noise(dec) * 2.0);
        dec = dec * (1.0 / FBM_MAG);
        amplitude *= FBM_MAG;
    }

    noise
}

pub fn height(mut p: Vec2) -> f32 {
    p -= vec2(-2000.0, 2000.0);

    let mut noise = fnoise(0.00003, p);

    noise -= p.magnitude2() * 0.000000004;
    noise.max(0.0)
}
