pub fn mid_point(a: egui::Pos2, b: egui::Pos2) -> egui::Pos2 {
    a + (b - a) / 2.0
}

/// ニュートン法で方程式 f(x) = 0 の解を求める
/// f: 関数, df: f の導関数, x0: 初期値, tol: 許容誤差, max_iter: 最大反復回数
fn newton_method(
    f: impl Fn(f32) -> f32,
    df: impl Fn(f32) -> f32,
    x0: f32,
    tol: f32,
    max_iter: usize,
) -> Option<f32> {
    let mut x = x0;

    for _ in 0..max_iter {
        let fx = f(x);
        let dfx = df(x);

        if dfx.abs() < 1e-6 {
            // 導関数が 0 に近いときは収束しないため終了
            return None;
        }

        let x_next = x - fx / dfx;

        if (x_next - x).abs() < tol {
            return Some(x_next); // 収束したら解を返す
        }

        x = x_next;
    }

    None // 最大反復回数に達した場合は解なし
}

/// ベジェ曲線と円の交点を計算
///
/// ### Parameters
/// - `bezier_start`: ベジェ曲線の始点
/// - `bezier_control`: ベジェ曲線の制御点
/// - `bezier_end`: ベジェ曲線の終点
/// - `circle_center`: 円の中心
/// - `circle_radius`: 円の半径
///
/// ### Returns
/// 交点が存在する場合はその座標と向き，存在しない場合は `None`
pub fn calc_intersection_of_bezier_and_circle(
    bezier_start: egui::Pos2,
    bezier_control: egui::Pos2,
    bezier_end: egui::Pos2,
    circle_center: egui::Pos2,
    circle_radius: f32,
) -> Option<(egui::Pos2, egui::Vec2)> {
    let (x0, y0) = (bezier_start.x, bezier_start.y);
    let (x1, y1) = (bezier_control.x, bezier_control.y);
    let (x2, y2) = (bezier_end.x, bezier_end.y);
    let (xc, yc) = (circle_center.x, circle_center.y);
    let r = circle_radius;

    let x =
        |t: f32| -> f32 { (1.0 - t).powf(2.0) * x0 + 2.0 * t * (1.0 - t) * x1 + t.powf(2.0) * x2 };
    let y =
        |t: f32| -> f32 { (1.0 - t).powf(2.0) * y0 + 2.0 * t * (1.0 - t) * y1 + t.powf(2.0) * y2 };

    let f = |t: f32| -> f32 { -r.powf(2.0) + (x(t) - xc).powf(2.0) + (y(t) - yc).powf(2.0) };

    let df = |t: f32| -> f32 {
        (-4.0 * t * x1 + 4.0 * t * x2 + 2.0 * x0 * (2.0 * t - 2.0) + 2.0 * x1 * (2.0 - 2.0 * t))
            * (t.powf(2.0) * x2 + t * x1 * (2.0 - 2.0 * t) + x0 * (1.0 - t).powf(2.0) - xc)
            + (-4.0 * t * y1
                + 4.0 * t * y2
                + 2.0 * y0 * (2.0 * t - 2.0)
                + 2.0 * y1 * (2.0 - 2.0 * t))
                * (t.powf(2.0) * y2 + t * y1 * (2.0 - 2.0 * t) + y0 * (1.0 - t).powf(2.0) - yc)
    };

    let t = newton_method(f, df, 0.5, 1e-6, 10)?;

    let dx = |t: f32| -> f32 { 2.0 * (x0 - 2.0 * x1 + x2) * t + 2.0 * (x1 - x0) };
    let dy = |t: f32| -> f32 { 2.0 * (y0 - 2.0 * y1 + y2) * t + 2.0 * (y1 - y0) };

    (0.0..1.0).contains(&t).then(|| {
        (
            egui::Pos2::new(x(t), y(t)),
            egui::Vec2::new(dx(t), dy(t)).normalized(),
        )
    })
}
