pub fn calc_bezier_control_point(
    start: egui::Pos2,
    end: egui::Pos2,
    distance: f32,
    is_clockwise: bool,
) -> egui::Pos2 {
    let mid = start + (end - start) / 2.0;
    let dir = (end - start).normalized().rot90();
    let control = mid + dir * if is_clockwise { distance } else { -distance };
    control
}

/// ベジェ曲線
pub fn bezier_curve(start: egui::Pos2, control: egui::Pos2, end: egui::Pos2, t: f32) -> egui::Pos2 {
    let x = (1.0 - t).powf(2.0) * start.x + 2.0 * t * (1.0 - t) * control.x + t.powf(2.0) * end.x;
    let y = (1.0 - t).powf(2.0) * start.y + 2.0 * t * (1.0 - t) * control.y + t.powf(2.0) * end.y;
    egui::Pos2::new(x, y)
}

/// ベジェ曲線のパラメータ`t`における微分
pub fn d_bezier_dt(start: egui::Pos2, control: egui::Pos2, end: egui::Pos2, t: f32) -> egui::Vec2 {
    let mt = 1.0 - t;
    let dx = 2.0 * mt * (control.x - start.x) + 2.0 * t * (end.x - control.x);
    let dy = 2.0 * mt * (control.y - start.y) + 2.0 * t * (end.y - control.y);
    egui::Vec2::new(dx, dy)
}

/// ベジェ曲線の2階微分
pub fn d2_bezier_dt2(start: egui::Pos2, control: egui::Pos2, end: egui::Pos2) -> egui::Vec2 {
    control.to_vec2() - 2.0 * start.to_vec2() + end.to_vec2()
}

/// ニュートン法で方程式 f(x) = 0 の解を求める
/// f: 関数, df: f の導関数, x0: 初期値, tol: 許容誤差, max_iter: 最大反復回数
pub fn newton_method(
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

    let bezier =
        |t: f32| -> egui::Pos2 { bezier_curve(bezier_start, bezier_control, bezier_end, t) };

    let f = |t: f32| -> f32 {
        let pos = bezier(t);
        -r.powf(2.0) + (pos.x - xc).powf(2.0) + (pos.y - yc).powf(2.0)
    };

    // df/dt (project://memo/intersection_of_bezier_and_circle.py で導出)
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

    (0.0..1.0).contains(&t).then(|| {
        (
            bezier(t),
            d_bezier_dt(bezier_start, bezier_control, bezier_end, t),
        )
    })
}
