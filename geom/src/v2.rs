use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[inline]
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

impl Vec2 {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const UNIT_X: Self = Self { x: 1.0, y: 0.0 };
    pub const UNIT_Y: Self = Self { x: 0.0, y: 1.0 };

    #[inline]
    pub fn perpendicular(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }

    #[inline]
    pub fn magnitude(self) -> f32 {
        self.magnitude2().sqrt()
    }

    #[inline]
    pub fn magnitude2(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    #[inline]
    pub fn perp_dot(self, rhs: Self) -> f32 {
        self.dot(rhs.perpendicular())
    }

    pub fn is_close(self, other: Self, close_dist: f32) -> bool {
        self.distance2(other) < close_dist * close_dist
    }

    #[inline]
    pub fn distance2(self, rhs: Self) -> f32 {
        (self - rhs).magnitude2()
    }

    #[inline]
    pub fn distance(self, rhs: Self) -> f32 {
        (self - rhs).magnitude()
    }

    /// Returns the angle between self and other in range [-pi; pi]
    #[inline]
    pub fn angle(self, other: Vec2) -> f32 {
        f32::atan2(Self::perp_dot(self, other), Self::dot(self, other))
    }

    #[inline]
    pub fn from_angle(angle: f32) -> Vec2 {
        Self {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    #[inline]
    pub fn try_normalize(self) -> Option<Vec2> {
        let m = self.magnitude();
        if m > 0.0 {
            Some(self / m)
        } else {
            None
        }
    }

    #[inline]
    pub fn flipy(self) -> Vec2 {
        Self {
            x: self.x,
            y: -self.y,
        }
    }

    #[inline]
    pub fn flipx(self) -> Vec2 {
        Self {
            x: -self.x,
            y: self.y,
        }
    }

    #[inline]
    pub fn normalize(self) -> Vec2 {
        let m = self.magnitude();
        self / m
    }

    #[inline]
    pub fn try_normalize_to(self, v: f32) -> Option<Vec2> {
        let m = self.magnitude();
        if m > 0.0 {
            Some(self * (v / m))
        } else {
            None
        }
    }

    #[inline]
    pub fn normalize_to(self, v: f32) -> Vec2 {
        let m = self.magnitude();
        self * (v / m)
    }

    #[inline]
    pub fn dir_dist(self) -> Option<(Vec2, f32)> {
        let m = self.magnitude();
        if m > 0.0 {
            Some((self / m, m))
        } else {
            None
        }
    }

    pub fn lerp(self, other: Self, coeff: f32) -> Self {
        self * (1.0 - coeff) + other * coeff
    }

    #[inline]
    pub fn min(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    #[inline]
    pub fn max(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    pub fn modulo(self, v: f32) -> Self {
        Self {
            x: self.x % v,
            y: self.y % v,
        }
    }

    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    pub fn fract(self) -> Self {
        Self {
            x: self.x.fract(),
            y: self.y.fract(),
        }
    }

    #[inline]
    pub fn cap_magnitude(self, max: f32) -> Vec2 {
        let m = self.magnitude();
        if m > max {
            self * max / m
        } else {
            self
        }
    }

    #[inline]
    pub fn approx_eq(self, other: Vec2) -> bool {
        let m = self.distance2(other);
        m < std::f32::EPSILON
    }

    #[inline]
    pub fn rotated_by(self, cossin: Vec2) -> Self {
        self.x * cossin - self.y * cossin.perpendicular()
    }
}

impl Eq for Vec2 {}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add for &Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<Vec2> for &Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<&Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: &Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub for &Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign for Vec2 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl Div<Vec2> for f32 {
    type Output = Vec2;

    fn div(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self / rhs.x,
            y: self / rhs.y,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<Vec2> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: Vec2) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl std::iter::Sum for Vec2 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut z = Vec2::ZERO;
        for x in iter {
            z += x;
        }
        z
    }
}

impl<'a> std::iter::Sum<&'a Vec2> for Vec2 {
    fn sum<I: Iterator<Item = &'a Vec2>>(iter: I) -> Self {
        let mut z = Vec2::ZERO;
        for &x in iter {
            z += x;
        }
        z
    }
}

impl DivAssign for Vec2 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl From<Vec2> for [f32; 2] {
    fn from(v: Vec2) -> Self {
        [v.x, v.y]
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(v: [f32; 2]) -> Self {
        Self { x: v[0], y: v[1] }
    }
}

impl From<Vec2> for mint::Point2<f32> {
    fn from(v: Vec2) -> Self {
        mint::Point2 { x: v.x, y: v.y }
    }
}

impl From<mint::Point2<f32>> for Vec2 {
    fn from(v: mint::Point2<f32>) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Vec2> for mint::Vector2<f32> {
    fn from(v: Vec2) -> Self {
        mint::Vector2 { x: v.x, y: v.y }
    }
}

impl From<mint::Vector2<f32>> for Vec2 {
    fn from(v: mint::Vector2<f32>) -> Self {
        Self { x: v.x, y: v.y }
    }
}
