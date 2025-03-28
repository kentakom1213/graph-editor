/// 可視化を行う
pub trait Visualize {
    /// グラフ G = (V,E) が与えられたとき，
    /// 頂点から 2 次元平面への写像 f: V → (0,1)^2 を構成する．
    fn resolve_vertex_position(&self, n: usize, edges: &[(usize, usize)]) -> Vec<egui::Vec2>;
}

pub mod visualize_methods {
    /// [0,1]^2 から一様ランダムにサンプリングする
    fn sample() -> egui::Vec2 {
        egui::vec2(rand::random::<f32>(), rand::random::<f32>())
    }

    /// 辺の重なりの回数を数える
    /// - 計算量: O(m^2)
    fn count_edge_crossing(positions: &[egui::Vec2], edges: &[(usize, usize)]) -> usize {
        let m = edges.len();
        let edge_segments = edges
            .iter()
            .map(|&(i, j)| {
                let p = positions[i];
                let q = positions[j];
                (p, q)
            })
            .collect::<Vec<_>>();

        let mut count = 0;

        for i in 0..m {
            for j in i + 1..m {
                let (p1, q1) = edge_segments[i];
                let (p2, q2) = edge_segments[j];

                if p1 == p2 || p1 == q2 || q1 == p2 || q1 == q2 {
                    continue;
                }
            }
        }

        count
    }

    /// 一様ランダムに各頂点の座標を選択する．
    pub struct Naive;

    impl super::Visualize for Naive {
        fn resolve_vertex_position(&self, n: usize, _edges: &[(usize, usize)]) -> Vec<egui::Vec2> {
            (0..n).map(|_| sample()).collect()
        }
    }

    /// 辺の重なりが減るように山登り法を用いて配置する．
    pub struct HillClimbing;

    impl super::Visualize for HillClimbing {
        fn resolve_vertex_position(&self, n: usize, edges: &[(usize, usize)]) -> Vec<egui::Vec2> {
            todo!()
        }
    }
}
