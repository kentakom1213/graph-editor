//! 2次元Affine変換
//!
//! 2次元平面上の点の拡大，縮小，平行移動

#![allow(clippy::needless_range_loop)]

use std::ops::{Mul, MulAssign};

use num_traits::One;

/// 2次元アフィン変換の0元行列
const AFFINE2D_ZERO: [[f32; 3]; 3] = [[0.0; 3]; 3];

/// 2次元アフィン変換の1元行列
const AFFINE2D_ONE: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// アフィン変換
#[derive(Debug, Clone, Copy)]
pub struct Affine2D(pub [[f32; 3]; 3]);

impl Affine2D {
    /// アフィン変換の平行移動
    pub fn from_transition(vec2: egui::Vec2) -> Self {
        Self([[1.0, 0.0, vec2.x], [0.0, 1.0, vec2.y], [0.0, 0.0, 1.0]])
    }

    /// アフィン変換の拡大縮小
    /// - `center`: 拡大縮小の中心
    /// - `scale`: 拡大縮小の倍率
    pub fn from_center_and_scale(center: egui::Pos2, scale: f32) -> Self {
        Self([
            [scale, 0.0, center.x * (1.0 - scale)],
            [0.0, scale, center.y * (1.0 - scale)],
            [0.0, 0.0, 1.0],
        ])
    }

    /// 並行移動成分を取得
    pub fn translation(&self) -> egui::Vec2 {
        egui::vec2(self.0[0][2], self.0[1][2])
    }

    /// スケールを取得
    pub fn scale_x(&self) -> f32 {
        self.0[0][0]
    }

    /// アフィン変換を合成する
    /// - scale の最小値，最大値の範囲を超えない操作のみ行う
    pub fn try_compose(&self, rhs: &Affine2D, scale_min: f32, scale_max: f32) -> Option<Affine2D> {
        let composed = *self * *rhs;
        let scale = composed.scale_x();

        (scale_min <= scale && scale <= scale_max).then_some(composed)
    }

    /// アフィン変換の逆元
    pub fn inverse(&self) -> Option<Self> {
        let (a, b, tx) = (self.0[0][0], self.0[0][1], self.0[0][2]);
        let (c, d, ty) = (self.0[1][0], self.0[1][1], self.0[1][2]);

        // 行列式を計算
        let det = a * d - b * c;
        if det.abs() < 1e-6 {
            // 正則でない場合
            return None;
        }

        let inv_det = 1.0 / det;

        // 線形部分の逆行列
        let a_inv = d * inv_det;
        let b_inv = -b * inv_det;
        let c_inv = -c * inv_det;
        let d_inv = a * inv_det;

        // 平行移動の逆変換
        let tx_inv = -(a_inv * tx + b_inv * ty);
        let ty_inv = -(c_inv * tx + d_inv * ty);

        Some(Self([
            [a_inv, b_inv, tx_inv],
            [c_inv, d_inv, ty_inv],
            [0.0, 0.0, 1.0],
        ]))
    }
}

impl Mul for Affine2D {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = AFFINE2D_ZERO;
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    res[i][j] += self.0[i][k] * rhs.0[k][j];
                }
            }
        }
        Self(res)
    }
}

impl MulAssign for Affine2D {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl One for Affine2D {
    fn one() -> Self {
        Self(AFFINE2D_ONE)
    }
}

/// アフィン変換を適用する
pub trait ApplyAffine: Sized {
    /// アフィン変換を適用した結果を取得する
    fn applied(&self, affine: &Affine2D) -> Self;
    /// アフィン変換を適用する
    fn apply(&mut self, affine: &Affine2D) {
        *self = self.applied(affine);
    }
}

impl ApplyAffine for egui::Vec2 {
    fn applied(&self, affine: &Affine2D) -> Self {
        let new_x = affine.0[0][0] * self.x + affine.0[0][1] * self.y + affine.0[0][2];
        let new_y = affine.0[1][0] * self.x + affine.0[1][1] * self.y + affine.0[1][2];
        egui::vec2(new_x, new_y)
    }
}

impl ApplyAffine for egui::Pos2 {
    fn applied(&self, affine: &Affine2D) -> Self {
        let new_x = affine.0[0][0] * self.x + affine.0[0][1] * self.y + affine.0[0][2];
        let new_y = affine.0[1][0] * self.x + affine.0[1][1] * self.y + affine.0[1][2];
        egui::pos2(new_x, new_y)
    }
}
