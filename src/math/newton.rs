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
