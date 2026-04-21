//! Lattice 시각화 도구
//!
//! Lattice 구조를 다양한 형식으로 시각화합니다.
//!
//! # 지원 형식
//!
//! - **DOT (Graphviz)**: 그래프 시각화 도구용 출력
//! - **HTML**: 인터랙티브 웹 뷰어
//! - **텍스트**: 디버깅용 덤프
//!
//! # Example
//!
//! ```rust,no_run
//! use mecab_ko_core::lattice::Lattice;
//! use mecab_ko_core::lattice_viz::{LatticeViz, VizFormat};
//!
//! let lattice = Lattice::new("한국어");
//! // ... 노드 추가 후 ...
//!
//! let viz = LatticeViz::new(&lattice);
//!
//! // DOT 형식 출력
//! let dot = viz.to_dot();
//! println!("{}", dot);
//!
//! // HTML 형식 출력
//! let html = viz.to_html();
//! std::fs::write("lattice.html", html).unwrap();
//! ```

use crate::lattice::{Lattice, Node, NodeType};
use std::fmt::Write;

/// 시각화 형식
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VizFormat {
    /// DOT/Graphviz 형식
    #[default]
    Dot,
    /// HTML 인터랙티브 뷰어
    Html,
    /// 텍스트 덤프
    Text,
    /// JSON 형식
    Json,
}

/// 시각화 옵션
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct VizOptions {
    /// 노드에 비용 표시
    pub show_cost: bool,
    /// 노드에 품사 표시
    pub show_pos: bool,
    /// 최적 경로 강조
    pub highlight_best_path: bool,
    /// 노드 타입별 색상 사용
    pub use_colors: bool,
    /// 최대 노드 수 (0이면 제한 없음)
    pub max_nodes: usize,
    /// 그래프 방향 (LR: 왼쪽→오른쪽, TB: 위→아래)
    pub direction: String,
}

impl Default for VizOptions {
    fn default() -> Self {
        Self {
            show_cost: true,
            show_pos: true,
            highlight_best_path: true,
            use_colors: true,
            max_nodes: 0,
            direction: "LR".to_string(),
        }
    }
}

impl VizOptions {
    /// 새 옵션 생성
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 비용 표시 설정
    #[must_use]
    pub const fn with_cost(mut self, show: bool) -> Self {
        self.show_cost = show;
        self
    }

    /// 품사 표시 설정
    #[must_use]
    pub const fn with_pos(mut self, show: bool) -> Self {
        self.show_pos = show;
        self
    }

    /// 최적 경로 강조 설정
    #[must_use]
    pub const fn with_best_path(mut self, highlight: bool) -> Self {
        self.highlight_best_path = highlight;
        self
    }

    /// 색상 사용 설정
    #[must_use]
    pub const fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }

    /// 최대 노드 수 설정
    #[must_use]
    pub const fn with_max_nodes(mut self, max: usize) -> Self {
        self.max_nodes = max;
        self
    }

    /// 그래프 방향 설정
    #[must_use]
    pub fn with_direction(mut self, dir: &str) -> Self {
        self.direction = dir.to_string();
        self
    }
}

/// Lattice 시각화 도구
pub struct LatticeViz<'a> {
    lattice: &'a Lattice,
    options: VizOptions,
    best_path_ids: Vec<u32>,
}

impl<'a> LatticeViz<'a> {
    /// 새 시각화 도구 생성
    #[must_use]
    pub fn new(lattice: &'a Lattice) -> Self {
        let best_path_ids = lattice.best_path().iter().map(|n| n.id).collect();
        Self {
            lattice,
            options: VizOptions::default(),
            best_path_ids,
        }
    }

    /// 옵션 설정
    #[must_use]
    pub fn with_options(mut self, options: VizOptions) -> Self {
        self.options = options;
        self
    }

    /// DOT 형식으로 변환
    #[must_use]
    pub fn to_dot(&self) -> String {
        let mut output = String::new();

        // 그래프 헤더
        writeln!(
            output,
            "digraph Lattice {{\n  rankdir={};\n  node [shape=box, fontname=\"Noto Sans KR\"];",
            self.options.direction
        )
        .ok();

        // 노드 정의
        self.write_dot_nodes(&mut output);

        // 엣지 정의
        self.write_dot_edges(&mut output);

        writeln!(output, "}}").ok();
        output
    }

    /// DOT 노드 정의 작성
    fn write_dot_nodes(&self, output: &mut String) {
        for node in self.lattice.nodes() {
            let is_on_best_path = self.best_path_ids.contains(&node.id);

            // 노드 스타일
            let (color, style) = self.get_node_style(node, is_on_best_path);

            // 노드 라벨
            let label = self.get_node_label(node);

            writeln!(
                output,
                "  n{} [label=\"{}\", fillcolor=\"{}\", style=\"{}\"];",
                node.id, label, color, style
            )
            .ok();
        }
    }

    /// DOT 엣지 정의 작성
    fn write_dot_edges(&self, output: &mut String) {
        let char_len = self.lattice.char_len();

        // 각 위치에서 끝나는 노드 → 시작하는 노드 연결
        for pos in 0..=char_len {
            let ending_nodes: Vec<_> = self.lattice.nodes_ending_at(pos).collect();
            let starting_nodes: Vec<_> = self.lattice.nodes_starting_at(pos).collect();

            for end_node in &ending_nodes {
                for start_node in &starting_nodes {
                    let is_best = self.best_path_ids.contains(&end_node.id)
                        && self.best_path_ids.contains(&start_node.id);

                    let style = if is_best && self.options.highlight_best_path {
                        "bold"
                    } else {
                        "solid"
                    };

                    let color = if is_best && self.options.highlight_best_path {
                        "red"
                    } else {
                        "black"
                    };

                    writeln!(
                        output,
                        "  n{} -> n{} [style={}, color={}];",
                        end_node.id, start_node.id, style, color
                    )
                    .ok();
                }
            }
        }
    }

    /// 노드 스타일 결정
    const fn get_node_style(
        &self,
        node: &Node,
        is_on_best_path: bool,
    ) -> (&'static str, &'static str) {
        if !self.options.use_colors {
            return ("white", "solid");
        }

        let base_color = match node.node_type {
            NodeType::Bos | NodeType::Eos => "#e0e0e0",
            NodeType::Known => "#d4edda",
            NodeType::Unknown => "#f8d7da",
            NodeType::User => "#d1ecf1",
        };

        let style = if is_on_best_path && self.options.highlight_best_path {
            "filled,bold"
        } else {
            "filled"
        };

        (base_color, style)
    }

    /// 노드 라벨 생성
    fn get_node_label(&self, node: &Node) -> String {
        let mut label = node.surface.to_string();

        if self.options.show_pos && !matches!(node.node_type, NodeType::Bos | NodeType::Eos) {
            let pos = node.feature.split(',').next().unwrap_or("*");
            write!(label, "\\n{pos}").ok();
        }

        if self.options.show_cost {
            if node.total_cost == i32::MAX {
                write!(label, "\\n[∞]").ok();
            } else {
                write!(label, "\\n[{}]", node.total_cost).ok();
            }
        }

        // DOT 특수문자 이스케이프
        label.replace('"', "\\\"")
    }

    /// HTML 형식으로 변환
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn to_html(&self) -> String {
        let dot = self.to_dot();
        let text = self.lattice.text();
        let graph_selector = "#lattice-graph";

        format!(
            r#"<!DOCTYPE html>
<html lang="ko">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Lattice Visualization: {text}</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script src="https://unpkg.com/@hpcc-js/wasm/dist/graphviz.umd.js"></script>
    <script src="https://unpkg.com/d3-graphviz@5.1.0/build/d3-graphviz.js"></script>
    <style>
        body {{
            font-family: 'Noto Sans KR', sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 100%;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            margin-top: 0;
        }}
        .info {{
            background: #f8f9fa;
            padding: 10px;
            border-radius: 4px;
            margin-bottom: 20px;
        }}
        .info span {{
            margin-right: 20px;
        }}
        .graph-container {{
            overflow: auto;
            border: 1px solid #ddd;
            border-radius: 4px;
            background: white;
        }}
        .legend {{
            margin-top: 20px;
            display: flex;
            gap: 20px;
            flex-wrap: wrap;
        }}
        .legend-item {{
            display: flex;
            align-items: center;
            gap: 5px;
        }}
        .legend-color {{
            width: 20px;
            height: 20px;
            border-radius: 4px;
            border: 1px solid #ccc;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Lattice Visualization</h1>
        <div class="info">
            <span><strong>입력:</strong> {text}</span>
            <span><strong>노드 수:</strong> {node_count}</span>
            <span><strong>문자 수:</strong> {char_len}</span>
        </div>
        <div id="lattice-graph" class="graph-container"></div>
        <div class="legend">
            <div class="legend-item">
                <div class="legend-color" style="background: #d4edda;"></div>
                <span>Known (사전)</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #f8d7da;"></div>
                <span>Unknown (미등록)</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #d1ecf1;"></div>
                <span>User (사용자)</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #e0e0e0;"></div>
                <span>BOS/EOS</span>
            </div>
        </div>
    </div>
    <script>
        const dot = `{dot_escaped}`;
        d3.select("{graph_selector}").graphviz()
            .zoom(true)
            .fit(true)
            .renderDot(dot);
    </script>
</body>
</html>"#,
            text = text,
            node_count = self.lattice.node_count(),
            char_len = self.lattice.char_len(),
            dot_escaped = dot.replace('`', "\\`").replace("${", "\\${"),
            graph_selector = graph_selector
        )
    }

    /// 텍스트 덤프 형식으로 변환
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut output = String::new();

        writeln!(output, "=== Lattice Dump ===").ok();
        writeln!(output, "Text: \"{}\"", self.lattice.text()).ok();
        writeln!(output, "Original: \"{}\"", self.lattice.original_text()).ok();
        writeln!(output, "Char length: {}", self.lattice.char_len()).ok();
        writeln!(output, "Node count: {}", self.lattice.node_count()).ok();
        writeln!(output).ok();

        // 노드 목록
        writeln!(output, "--- Nodes ---").ok();
        for node in self.lattice.nodes() {
            let type_str = match node.node_type {
                NodeType::Bos => "BOS",
                NodeType::Eos => "EOS",
                NodeType::Known => "KNW",
                NodeType::Unknown => "UNK",
                NodeType::User => "USR",
            };

            let is_best = if self.best_path_ids.contains(&node.id) {
                " *"
            } else {
                ""
            };

            writeln!(
                output,
                "[{:3}] {} {:>4} {:<10} ({:2}-{:2}) cost={:6} total={:10} prev={:3}{}",
                node.id,
                type_str,
                node.feature.split(',').next().unwrap_or("*"),
                node.surface,
                node.start_pos,
                node.end_pos,
                node.word_cost,
                if node.total_cost == i32::MAX {
                    "∞".to_string()
                } else {
                    node.total_cost.to_string()
                },
                if node.prev_node_id == u32::MAX {
                    "-".to_string()
                } else {
                    node.prev_node_id.to_string()
                },
                is_best
            )
            .ok();
        }

        // 위치별 노드
        writeln!(output).ok();
        writeln!(output, "--- By Position ---").ok();
        for pos in 0..=self.lattice.char_len() {
            let ending: Vec<_> = self.lattice.nodes_ending_at(pos).collect();
            let starting: Vec<_> = self.lattice.nodes_starting_at(pos).collect();

            if !ending.is_empty() || !starting.is_empty() {
                writeln!(output, "Position {pos}:").ok();
                if !ending.is_empty() {
                    let names: Vec<_> = ending
                        .iter()
                        .map(|n| format!("{}({})", n.surface, n.id))
                        .collect();
                    writeln!(output, "  Ending: {}", names.join(", ")).ok();
                }
                if !starting.is_empty() {
                    let names: Vec<_> = starting
                        .iter()
                        .map(|n| format!("{}({})", n.surface, n.id))
                        .collect();
                    writeln!(output, "  Starting: {}", names.join(", ")).ok();
                }
            }
        }

        // 최적 경로
        if !self.best_path_ids.is_empty() {
            writeln!(output).ok();
            writeln!(output, "--- Best Path ---").ok();
            let path: Vec<_> = self
                .lattice
                .best_path()
                .iter()
                .map(|n| format!("{}", n.surface))
                .collect();
            writeln!(output, "{}", path.join(" → ")).ok();
        }

        output
    }

    /// JSON 형식으로 변환
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // 노드 정보 수집
        for node in self.lattice.nodes() {
            let node_type = match node.node_type {
                NodeType::Bos => "bos",
                NodeType::Eos => "eos",
                NodeType::Known => "known",
                NodeType::Unknown => "unknown",
                NodeType::User => "user",
            };

            let pos = node.feature.split(',').next().unwrap_or("*");
            let is_best = self.best_path_ids.contains(&node.id);

            nodes.push(format!(
                r#"    {{
      "id": {},
      "surface": "{}",
      "pos": "{}",
      "type": "{}",
      "start": {},
      "end": {},
      "wordCost": {},
      "totalCost": {},
      "isBestPath": {}
    }}"#,
                node.id,
                node.surface.replace('"', "\\\""),
                pos,
                node_type,
                node.start_pos,
                node.end_pos,
                node.word_cost,
                if node.total_cost == i32::MAX {
                    "null".to_string()
                } else {
                    node.total_cost.to_string()
                },
                is_best
            ));
        }

        // 엣지 정보 수집
        let char_len = self.lattice.char_len();
        for pos in 0..=char_len {
            let ending_nodes: Vec<_> = self.lattice.nodes_ending_at(pos).collect();
            let starting_nodes: Vec<_> = self.lattice.nodes_starting_at(pos).collect();

            for end_node in &ending_nodes {
                for start_node in &starting_nodes {
                    let is_best = self.best_path_ids.contains(&end_node.id)
                        && self.best_path_ids.contains(&start_node.id);

                    edges.push(format!(
                        r#"    {{"from": {}, "to": {}, "isBestPath": {}}}"#,
                        end_node.id, start_node.id, is_best
                    ));
                }
            }
        }

        format!(
            r#"{{
  "text": "{}",
  "originalText": "{}",
  "charLength": {},
  "nodes": [
{}
  ],
  "edges": [
{}
  ]
}}"#,
            self.lattice.text().replace('"', "\\\""),
            self.lattice.original_text().replace('"', "\\\""),
            self.lattice.char_len(),
            nodes.join(",\n"),
            edges.join(",\n")
        )
    }

    /// 지정된 형식으로 변환
    #[must_use]
    pub fn format(&self, fmt: VizFormat) -> String {
        match fmt {
            VizFormat::Dot => self.to_dot(),
            VizFormat::Html => self.to_html(),
            VizFormat::Text => self.to_text(),
            VizFormat::Json => self.to_json(),
        }
    }
}

/// 편의 함수: Lattice를 DOT 형식으로 변환
#[must_use]
pub fn lattice_to_dot(lattice: &Lattice) -> String {
    LatticeViz::new(lattice).to_dot()
}

/// 편의 함수: Lattice를 HTML 형식으로 변환
#[must_use]
pub fn lattice_to_html(lattice: &Lattice) -> String {
    LatticeViz::new(lattice).to_html()
}

/// 편의 함수: Lattice를 텍스트 덤프로 변환
#[must_use]
pub fn lattice_to_text(lattice: &Lattice) -> String {
    LatticeViz::new(lattice).to_text()
}

/// 편의 함수: Lattice를 JSON 형식으로 변환
#[must_use]
pub fn lattice_to_json(lattice: &Lattice) -> String {
    LatticeViz::new(lattice).to_json()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lattice::NodeBuilder;

    fn create_test_lattice() -> Lattice {
        let mut lattice = Lattice::new("안녕");

        lattice.add_node(
            NodeBuilder::new("안녕", 0, 2)
                .left_id(1)
                .right_id(1)
                .word_cost(1000)
                .feature("NNG,*,F,안녕,*,*,*,*"),
        );

        lattice.add_node(
            NodeBuilder::new("안", 0, 1)
                .left_id(2)
                .right_id(2)
                .word_cost(2000)
                .feature("NNG,*,T,안,*,*,*,*"),
        );

        lattice.add_node(
            NodeBuilder::new("녕", 1, 2)
                .left_id(3)
                .right_id(3)
                .word_cost(3000)
                .feature("NNG,*,T,녕,*,*,*,*"),
        );

        lattice
    }

    #[test]
    fn test_to_dot() {
        let lattice = create_test_lattice();
        let viz = LatticeViz::new(&lattice);
        let dot = viz.to_dot();

        assert!(dot.contains("digraph Lattice"));
        assert!(dot.contains("안녕"));
        assert!(dot.contains("->"));
    }

    #[test]
    fn test_to_text() {
        let lattice = create_test_lattice();
        let viz = LatticeViz::new(&lattice);
        let text = viz.to_text();

        assert!(text.contains("Lattice Dump"));
        assert!(text.contains("안녕"));
        assert!(text.contains("By Position"));
    }

    #[test]
    fn test_to_json() {
        let lattice = create_test_lattice();
        let viz = LatticeViz::new(&lattice);
        let json = viz.to_json();

        assert!(json.contains("\"text\": \"안녕\""));
        assert!(json.contains("\"nodes\""));
        assert!(json.contains("\"edges\""));
    }

    #[test]
    fn test_to_html() {
        let lattice = create_test_lattice();
        let viz = LatticeViz::new(&lattice);
        let html = viz.to_html();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Lattice Visualization"));
        assert!(html.contains("d3-graphviz"));
    }

    #[test]
    fn test_viz_options() {
        let options = VizOptions::new()
            .with_cost(false)
            .with_pos(false)
            .with_best_path(false)
            .with_colors(false);

        assert!(!options.show_cost);
        assert!(!options.show_pos);
        assert!(!options.highlight_best_path);
        assert!(!options.use_colors);
    }

    #[test]
    fn test_format_selection() {
        let lattice = create_test_lattice();
        let viz = LatticeViz::new(&lattice);

        let dot = viz.format(VizFormat::Dot);
        let text = viz.format(VizFormat::Text);
        let json = viz.format(VizFormat::Json);
        let html = viz.format(VizFormat::Html);

        assert!(dot.contains("digraph"));
        assert!(text.contains("Lattice Dump"));
        assert!(json.contains("\"nodes\""));
        assert!(html.contains("<!DOCTYPE html>"));
    }
}
