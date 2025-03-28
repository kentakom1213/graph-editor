/// 可視化を行う
pub trait Visualize {
    /// グラフ G = (V,E) が与えられたとき，
    /// 頂点から 2 次元平面への写像 f: V → (0,1)^2 を構成する．
    fn resolve_vertex_position(&self, n: usize, edges: &[(usize, usize)]) -> Vec<egui::Vec2>;
}

pub mod visualize_methods {
    /// [0,1]^2 から一様ランダムにサンプリングする
    fn sample_point() -> egui::Vec2 {
        egui::vec2(rand::random::<f32>(), rand::random::<f32>())
    }

    /// 2 点の外積を計算する
    fn cross(a: egui::Vec2, b: egui::Vec2) -> f32 {
        a.x * b.y - a.y * b.x
    }

    /// 線分同士の交差判定
    fn is_crossing((p1, q1): (egui::Vec2, egui::Vec2), (p2, q2): (egui::Vec2, egui::Vec2)) -> bool {
        let d1 = cross(q1 - p1, p2 - p1);
        let d2 = cross(q1 - p1, q2 - p1);
        let d3 = cross(q2 - p2, p1 - p2);
        let d4 = cross(q2 - p2, q1 - p2);

        d1 * d2 < 0.0 && d3 * d4 < 0.0
    }

    /// 辺の重なりの回数を数える
    /// - 計算量: O(m^2)
    fn count_edge_crossing(positions: &[egui::Vec2], edges: &[(usize, usize)]) -> usize {
        let m = edges.len();

        let mut count = 0;

        for i in 0..m {
            for j in i + 1..m {
                let (u, v) = edges[i];
                let (w, x) = edges[j];

                let p1 = positions[u];
                let q1 = positions[v];
                let p2 = positions[w];
                let q2 = positions[x];

                if is_crossing((p1, q1), (p2, q2)) {
                    count += 1;
                }
            }
        }

        count
    }

    // -------------------- Visualize Methods --------------------
    /// 一様ランダムに各頂点の座標を選択する．
    pub struct Naive;

    impl super::Visualize for Naive {
        fn resolve_vertex_position(&self, n: usize, _edges: &[(usize, usize)]) -> Vec<egui::Vec2> {
            (0..n).map(|_| sample_point()).collect()
        }
    }

    /// 辺の重なりが減るように山登り法を用いて配置する．
    /// - `max_iter`: 最大反復回数
    pub struct HillClimbing(pub usize);

    impl super::Visualize for HillClimbing {
        fn resolve_vertex_position(&self, n: usize, edges: &[(usize, usize)]) -> Vec<egui::Vec2> {
            let initial_positions = (0..n).map(|_| sample_point()).collect::<Vec<_>>();

            let mut best_positions = initial_positions.clone();
            let mut best_crossing = count_edge_crossing(&best_positions, edges);

            for _ in 0..self.0 {
                let mut new_positions = best_positions.clone();

                // 1 つの頂点をランダムに選択して座標を変更する
                let i = rand::random::<usize>() % n;
                new_positions[i] = sample_point();

                // 辺の重なりが減るような配置を選択する
                let new_crossing = count_edge_crossing(&new_positions, edges);

                if new_crossing < best_crossing {
                    best_positions = new_positions;
                    best_crossing = new_crossing;
                }
            }

            best_positions
        }
    }
}
