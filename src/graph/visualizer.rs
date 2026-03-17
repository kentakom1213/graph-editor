/// 可視化を行う
pub trait Visualizer {
    /// グラフ G = (V,E) が与えられたとき，
    /// 頂点から 2 次元平面への写像 f: V → (0,1)^2 を構成する．
    fn resolve_vertex_position(&self, n: usize, edges: &[(usize, usize)]) -> Vec<egui::Vec2>;
}

pub mod visualize_methods {
    #![allow(dead_code)]

    const JACOBI_EPS: f64 = 1e-10;
    const JACOBI_MAX_ITER_FACTOR: usize = 20;
    const AXIS_EPS: f32 = 1e-6;

    /// [0,1]^2 から一様ランダムにサンプリングする
    fn sample_point() -> egui::Vec2 {
        egui::vec2(rand::random::<f32>(), rand::random::<f32>())
    }

    fn circular_layout(n: usize) -> Vec<egui::Vec2> {
        if n == 0 {
            return vec![];
        }

        if n == 1 {
            return vec![egui::vec2(0.5, 0.5)];
        }

        (0..n)
            .map(|i| {
                let theta = std::f32::consts::TAU * i as f32 / n as f32;
                egui::vec2(theta.cos(), theta.sin()) * 0.4 + egui::vec2(0.5, 0.5)
            })
            .collect()
    }

    fn build_laplacian(n: usize, edges: &[(usize, usize)]) -> Vec<Vec<f64>> {
        let mut laplacian = vec![vec![0.0; n]; n];

        for &(u, v) in edges {
            if u >= n || v >= n || u == v {
                continue;
            }

            laplacian[u][u] += 1.0;
            laplacian[v][v] += 1.0;
            laplacian[u][v] -= 1.0;
            laplacian[v][u] -= 1.0;
        }

        laplacian
    }

    fn jacobi_eigendecomposition(mut a: Vec<Vec<f64>>) -> (Vec<f64>, Vec<Vec<f64>>) {
        let n = a.len();
        let mut eigenvectors = vec![vec![0.0; n]; n];

        for (i, row) in eigenvectors.iter_mut().enumerate() {
            row[i] = 1.0;
        }

        if n <= 1 {
            let eigenvalues = a
                .into_iter()
                .enumerate()
                .map(|(i, row)| row[i])
                .collect::<Vec<_>>();
            return (eigenvalues, eigenvectors);
        }

        let max_iter = JACOBI_MAX_ITER_FACTOR * n * n;

        for _ in 0..max_iter {
            let mut p = 0;
            let mut q = 1;
            let mut max_off_diag = 0.0;

            for i in 0..n {
                for j in i + 1..n {
                    let value = a[i][j].abs();
                    if value > max_off_diag {
                        max_off_diag = value;
                        p = i;
                        q = j;
                    }
                }
            }

            if max_off_diag < JACOBI_EPS {
                break;
            }

            let app = a[p][p];
            let aqq = a[q][q];
            let apq = a[p][q];
            let phi = 0.5 * (2.0 * apq).atan2(aqq - app);
            let c = phi.cos();
            let s = phi.sin();

            for i in 0..n {
                if i == p || i == q {
                    continue;
                }

                let aip = a[i][p];
                let aiq = a[i][q];
                a[i][p] = c * aip - s * aiq;
                a[p][i] = a[i][p];
                a[i][q] = s * aip + c * aiq;
                a[q][i] = a[i][q];
            }

            a[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
            a[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
            a[p][q] = 0.0;
            a[q][p] = 0.0;

            for row in &mut eigenvectors {
                let vip = row[p];
                let viq = row[q];
                row[p] = c * vip - s * viq;
                row[q] = s * vip + c * viq;
            }
        }

        let eigenvalues = (0..n).map(|i| a[i][i]).collect::<Vec<_>>();
        (eigenvalues, eigenvectors)
    }

    fn normalize_axis(axis: &[f64]) -> Option<Vec<f32>> {
        if axis.is_empty() {
            return Some(vec![]);
        }

        let mean = axis.iter().sum::<f64>() / axis.len() as f64;
        let centered = axis.iter().map(|v| v - mean).collect::<Vec<_>>();
        let max_abs = centered.iter().map(|v| v.abs()).fold(0.0_f64, f64::max);

        if max_abs < AXIS_EPS as f64 {
            return None;
        }

        Some(
            centered
                .into_iter()
                .map(|v| (0.5 + 0.5 * (v / max_abs)) as f32)
                .collect(),
        )
    }

    fn spectral_layout(n: usize, edges: &[(usize, usize)]) -> Vec<egui::Vec2> {
        if n <= 1 {
            return circular_layout(n);
        }

        let laplacian = build_laplacian(n, edges);
        let (eigenvalues, eigenvectors) = jacobi_eigendecomposition(laplacian);

        let mut order = (0..n).collect::<Vec<_>>();
        order.sort_by(|&lhs, &rhs| eigenvalues[lhs].total_cmp(&eigenvalues[rhs]));

        let x_axis = order
            .get(1)
            .map(|&index| {
                eigenvectors
                    .iter()
                    .map(|row| row[index])
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![0.0; n]);
        let y_axis = order
            .get(2)
            .map(|&index| {
                eigenvectors
                    .iter()
                    .map(|row| row[index])
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![0.0; n]);

        let xs = normalize_axis(&x_axis);
        let ys = normalize_axis(&y_axis);

        match (xs, ys) {
            (Some(xs), Some(ys)) => xs
                .into_iter()
                .zip(ys)
                .map(|(x, y)| egui::vec2(x, y))
                .collect(),
            (Some(xs), None) => xs
                .into_iter()
                .enumerate()
                .map(|(i, x)| {
                    let y = if n == 1 {
                        0.5
                    } else {
                        0.2 + 0.6 * i as f32 / (n - 1) as f32
                    };
                    egui::vec2(x, y)
                })
                .collect(),
            _ => circular_layout(n),
        }
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

    // -------------------- Visualizer Methods --------------------
    /// 一様ランダムに各頂点の座標を選択する．
    pub struct Naive;

    impl super::Visualizer for Naive {
        fn resolve_vertex_position(&self, n: usize, _edges: &[(usize, usize)]) -> Vec<egui::Vec2> {
            (0..n).map(|_| sample_point()).collect()
        }
    }

    /// ラプラシアンの第2・第3固有ベクトルから初期配置を構成する．
    pub struct Spectral;

    impl super::Visualizer for Spectral {
        fn resolve_vertex_position(&self, n: usize, edges: &[(usize, usize)]) -> Vec<egui::Vec2> {
            spectral_layout(n, edges)
        }
    }

    /// 辺の重なりが減るように山登り法を用いて配置する．
    /// - `max_iter`: 最大反復回数
    pub struct HillClimbing(pub usize);

    impl super::Visualizer for HillClimbing {
        fn resolve_vertex_position(&self, n: usize, edges: &[(usize, usize)]) -> Vec<egui::Vec2> {
            if n == 0 {
                return vec![];
            }

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

    /// 焼きなまし法で辺の重なりを減らすように配置する．
    /// - `max_iter`: 最大反復回数
    /// - `initial_temp`: 初期温度
    /// - `cooling_rate`: 温度の減衰率
    pub struct SimulatedAnnealing {
        pub max_iter: usize,
        pub initial_temp: f32,
        pub cooling_rate: f32,
    }

    impl super::Visualizer for SimulatedAnnealing {
        fn resolve_vertex_position(&self, n: usize, edges: &[(usize, usize)]) -> Vec<egui::Vec2> {
            if n == 0 {
                return vec![];
            }

            if self.max_iter == 0 {
                return (0..n).map(|_| sample_point()).collect();
            }

            let mut current_positions = (0..n).map(|_| sample_point()).collect::<Vec<_>>();
            let mut current_crossing = count_edge_crossing(&current_positions, edges);

            let mut best_positions = current_positions.clone();
            let mut best_crossing = current_crossing;

            let mut temperature = self.initial_temp.max(0.0);
            let cooling_rate = self.cooling_rate;

            for _ in 0..self.max_iter {
                let mut new_positions = current_positions.clone();

                // 1 つの頂点をランダムに選択して座標を変更する
                let i = rand::random::<usize>() % n;
                new_positions[i] = sample_point();

                let new_crossing = count_edge_crossing(&new_positions, edges);
                let delta = new_crossing as isize - current_crossing as isize;

                let accept = if delta <= 0 {
                    true
                } else if temperature > 0.0 {
                    let prob = (-(delta as f32) / temperature).exp();
                    rand::random::<f32>() < prob
                } else {
                    false
                };

                if accept {
                    current_positions = new_positions;
                    current_crossing = new_crossing;

                    if current_crossing < best_crossing {
                        best_positions = current_positions.clone();
                        best_crossing = current_crossing;
                    }
                }

                temperature *= cooling_rate;
            }

            best_positions
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{spectral_layout, Spectral};
        use crate::graph::Visualizer;

        #[test]
        fn spectral_layout_stays_inside_unit_square() {
            let positions = spectral_layout(4, &[(0, 1), (1, 2), (2, 3)]);

            assert_eq!(positions.len(), 4);
            assert!(positions.iter().all(|p| (0.0..=1.0).contains(&p.x)));
            assert!(positions.iter().all(|p| (0.0..=1.0).contains(&p.y)));
        }

        #[test]
        fn spectral_visualizer_handles_disconnected_graph() {
            let visualizer = Spectral;
            let positions = visualizer.resolve_vertex_position(4, &[(0, 1), (2, 3)]);

            assert_eq!(positions.len(), 4);
            assert!(positions.windows(2).any(|w| w[0] != w[1]));
        }
    }
}
