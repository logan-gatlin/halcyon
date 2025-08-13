#![allow(dead_code)]

pub struct Graph {
  pub nodes: Vec<Node>,
  pub edges: Vec<Edge>,
}

impl Graph {
  pub fn new() -> Self {
    Self {
      nodes: vec![],
      edges: vec![],
    }
  }

  pub fn new_node(&mut self, id: String, content: String) {
    self.nodes.push(Node { id, content });
  }

  pub fn new_edge(&mut self, source: String, target: String) {
    self.edges.push(Edge { source, target });
  }

  pub fn to_json(&self) -> String {
    let nodes = self.nodes.iter().map(|n| {
      format!(
        r#"{{ "data": {{ "id": "{}", "label": "{}" }} }}"#,
        n.id, n.content
      )
    });

    let edges = self.edges.iter().map(|e| {
      format!(
        r#"{{ "data": {{ "source": "{}", "target": "{}" }}, "group": "edges" }}"#,
        e.source, e.target
      )
    });
    format!("[{}]", nodes.chain(edges).collect::<Vec<_>>().join(",\n"))
  }
}

pub struct Node {
  pub id: String,
  pub content: String,
}

pub struct Edge {
  pub source: String,
  pub target: String,
}
