/// 可視化を行う
pub trait Visualize {
    /// グラフ G = (V,E) が与えられたとき，
    /// 頂点から 2 次元平面への写像 f: V → (0,1)^2 を構成する．
    fn resolve_vertex_position(&self, n: usize, edges: &[(usize, usize)]) -> Vec<egui::Pos2>;
}

/// 一様ランダムに各頂点の座標を選択する．
pub struct Naive;

impl Visualize for Naive {
    fn resolve_vertex_position(&self, n: usize, _edges: &[(usize, usize)]) -> Vec<egui::Pos2> {
        (0..n)
            .map(|_| egui::pos2(rand::random::<f32>(), rand::random::<f32>()))
            .collect()
    }
}
