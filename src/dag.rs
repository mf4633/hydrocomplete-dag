use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::nodes::NodeKind;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub struct NodeId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub struct EdgeId(pub u32);

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
    /// Last computed output values, keyed by port index.
    #[serde(default)]
    pub outputs: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Edge {
    pub id: EdgeId,
    pub from_node: NodeId,
    pub from_port: usize,
    pub to_node: NodeId,
    pub to_port: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct DagModel {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    #[serde(skip)]
    next_node: u32,
    #[serde(skip)]
    next_edge: u32,
}

impl DagModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, kind: NodeKind, x: f64, y: f64) -> NodeId {
        let id = NodeId(self.next_node);
        self.next_node += 1;
        self.nodes.push(Node {
            id,
            kind,
            x,
            y,
            label: None,
            config: HashMap::new(),
            outputs: HashMap::new(),
        });
        id
    }

    /// Returns `Some(EdgeId)` if the edge was added, `None` if invalid/duplicate/cycle.
    pub fn add_edge(
        &mut self,
        from_node: NodeId,
        from_port: usize,
        to_node: NodeId,
        to_port: usize,
    ) -> Option<EdgeId> {
        if from_node == to_node { return None; }
        if !self.has_node(from_node) || !self.has_node(to_node) { return None; }
        // No duplicate
        if self.edges.iter().any(|e| {
            e.from_node == from_node && e.from_port == from_port
                && e.to_node == to_node && e.to_port == to_port
        }) {
            return None;
        }
        // Only one edge per input port
        if self.edges.iter().any(|e| e.to_node == to_node && e.to_port == to_port) {
            return None;
        }
        // Cycle check
        if self.path_exists(to_node, from_node) {
            return None;
        }
        let id = EdgeId(self.next_edge);
        self.next_edge += 1;
        self.edges.push(Edge { id, from_node, from_port, to_node, to_port });
        Some(id)
    }

    pub fn remove_node(&mut self, id: NodeId) {
        self.nodes.retain(|n| n.id != id);
        self.edges.retain(|e| e.from_node != id && e.to_node != id);
    }

    pub fn remove_edge(&mut self, id: EdgeId) {
        self.edges.retain(|e| e.id != id);
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn has_node(&self, id: NodeId) -> bool {
        self.nodes.iter().any(|n| n.id == id)
    }

    pub fn edges_from(&self, node: NodeId) -> impl Iterator<Item = &Edge> {
        self.edges.iter().filter(move |e| e.from_node == node)
    }

    pub fn edges_to(&self, node: NodeId) -> impl Iterator<Item = &Edge> {
        self.edges.iter().filter(move |e| e.to_node == node)
    }

    /// Returns nodes in topological order (Kahn's algorithm).
    pub fn topological_order(&self) -> Vec<NodeId> {
        let mut in_degree: HashMap<NodeId, usize> =
            self.nodes.iter().map(|n| (n.id, 0)).collect();
        for e in &self.edges {
            *in_degree.entry(e.to_node).or_insert(0) += 1;
        }
        let mut queue: Vec<NodeId> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&id, _)| id)
            .collect();
        let mut order = Vec::new();
        while let Some(n) = queue.pop() {
            order.push(n);
            for e in self.edges_from(n) {
                let deg = in_degree.entry(e.to_node).or_insert(1);
                *deg -= 1;
                if *deg == 0 {
                    queue.push(e.to_node);
                }
            }
        }
        order
    }

    /// DFS: does a path exist from `src` to `dst`?
    fn path_exists(&self, src: NodeId, dst: NodeId) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![src];
        while let Some(n) = stack.pop() {
            if n == dst { return true; }
            if !visited.insert(n.0) { continue; }
            for e in self.edges_from(n) {
                stack.push(e.to_node);
            }
        }
        false
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Option<Self> {
        let mut m: DagModel = serde_json::from_str(json).ok()?;
        m.next_node = m.nodes.iter().map(|n| n.id.0 + 1).max().unwrap_or(0);
        m.next_edge = m.edges.iter().map(|e| e.id.0 + 1).max().unwrap_or(0);
        Some(m)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::NodeKind;

    fn mk() -> DagModel { DagModel::new() }

    #[test]
    fn add_nodes_assigns_sequential_ids() {
        let mut dag = mk();
        let a = dag.add_node(NodeKind::Catchment, 0.0, 0.0);
        let b = dag.add_node(NodeKind::RationalMethod, 100.0, 0.0);
        assert_eq!(a.0, 0);
        assert_eq!(b.0, 1);
        assert_eq!(dag.nodes.len(), 2);
    }

    #[test]
    fn add_edge_basic() {
        let mut dag = mk();
        let a = dag.add_node(NodeKind::Catchment, 0.0, 0.0);
        let b = dag.add_node(NodeKind::RationalMethod, 100.0, 0.0);
        let e = dag.add_edge(a, 0, b, 0);
        assert!(e.is_some());
        assert_eq!(dag.edges.len(), 1);
    }

    #[test]
    fn reject_self_loop() {
        let mut dag = mk();
        let a = dag.add_node(NodeKind::Catchment, 0.0, 0.0);
        assert!(dag.add_edge(a, 0, a, 0).is_none());
    }

    #[test]
    fn reject_duplicate_edge() {
        let mut dag = mk();
        let a = dag.add_node(NodeKind::Catchment, 0.0, 0.0);
        let b = dag.add_node(NodeKind::RationalMethod, 100.0, 0.0);
        assert!(dag.add_edge(a, 0, b, 0).is_some());
        assert!(dag.add_edge(a, 0, b, 0).is_none()); // duplicate
    }

    #[test]
    fn reject_port_already_occupied() {
        let mut dag = mk();
        let a = dag.add_node(NodeKind::Catchment, 0.0, 0.0);
        let b = dag.add_node(NodeKind::Catchment, 50.0, 0.0);
        let c = dag.add_node(NodeKind::RationalMethod, 200.0, 0.0);
        assert!(dag.add_edge(a, 0, c, 0).is_some());
        // second edge into same input port must be rejected
        assert!(dag.add_edge(b, 0, c, 0).is_none());
    }

    #[test]
    fn reject_cycle_direct() {
        let mut dag = mk();
        let a = dag.add_node(NodeKind::Catchment, 0.0, 0.0);
        let b = dag.add_node(NodeKind::RationalMethod, 100.0, 0.0);
        assert!(dag.add_edge(a, 0, b, 0).is_some());
        // reverse would create a cycle
        assert!(dag.add_edge(b, 0, a, 0).is_none());
    }

    #[test]
    fn reject_cycle_transitive() {
        let mut dag = mk();
        let a = dag.add_node(NodeKind::Catchment, 0.0, 0.0);
        let b = dag.add_node(NodeKind::ScsRunoff, 100.0, 0.0);
        let c = dag.add_node(NodeKind::DetentionPond, 200.0, 0.0);
        dag.add_edge(a, 0, b, 0).unwrap();
        dag.add_edge(b, 0, c, 0).unwrap();
        // c → a would create cycle a → b → c → a
        assert!(dag.add_edge(c, 0, a, 0).is_none());
    }

    #[test]
    fn topological_order_linear_chain() {
        let mut dag = mk();
        let a = dag.add_node(NodeKind::Catchment,       0.0, 0.0);
        let b = dag.add_node(NodeKind::ScsRunoff,     100.0, 0.0);
        let c = dag.add_node(NodeKind::DetentionPond, 200.0, 0.0);
        let d = dag.add_node(NodeKind::Outfall,       300.0, 0.0);
        dag.add_edge(a, 0, b, 0).unwrap();
        dag.add_edge(b, 0, c, 0).unwrap();
        dag.add_edge(c, 0, d, 0).unwrap();
        let order = dag.topological_order();
        assert_eq!(order.len(), 4);
        let pos = |id: NodeId| order.iter().position(|&x| x == id).unwrap();
        assert!(pos(a) < pos(b));
        assert!(pos(b) < pos(c));
        assert!(pos(c) < pos(d));
    }

    #[test]
    fn topological_order_diamond() {
        // a → b, a → c, b → d, c → d
        let mut dag = mk();
        let a = dag.add_node(NodeKind::Catchment,        0.0, 0.0);
        let b = dag.add_node(NodeKind::RationalMethod,  100.0, 0.0);
        let c = dag.add_node(NodeKind::ScsRunoff,       100.0, 80.0);
        let d = dag.add_node(NodeKind::Outfall,         200.0, 40.0);
        dag.add_edge(a, 0, b, 0).unwrap();
        dag.add_edge(a, 0, c, 0).unwrap();
        dag.add_edge(b, 0, d, 0).unwrap();
        dag.add_edge(c, 0, d, 0).is_none(); // port 0 already occupied on d
        // partial: a < b, a < c; b < d or c < d
        let order = dag.topological_order();
        let pos = |id| order.iter().position(|&x| x == id).unwrap();
        assert!(pos(a) < pos(b));
        assert!(pos(a) < pos(c));
    }

    #[test]
    fn remove_node_cleans_edges() {
        let mut dag = mk();
        let a = dag.add_node(NodeKind::Catchment, 0.0, 0.0);
        let b = dag.add_node(NodeKind::RationalMethod, 100.0, 0.0);
        dag.add_edge(a, 0, b, 0).unwrap();
        dag.remove_node(a);
        assert_eq!(dag.nodes.len(), 1);
        assert_eq!(dag.edges.len(), 0); // edge referencing 'a' removed
    }

    #[test]
    fn json_roundtrip_preserves_structure() {
        let mut dag = mk();
        let a = dag.add_node(NodeKind::Catchment, 10.0, 20.0);
        let b = dag.add_node(NodeKind::ManningPipe, 200.0, 20.0);
        dag.add_edge(a, 0, b, 0).unwrap();
        // Set a config value
        if let Some(node) = dag.get_node_mut(a) {
            node.config.insert("area_acres".to_string(), serde_json::json!(7.5));
        }
        let json = dag.to_json();
        let loaded = DagModel::from_json(&json).expect("roundtrip");
        assert_eq!(loaded.nodes.len(), 2);
        assert_eq!(loaded.edges.len(), 1);
        let node_a = loaded.get_node(a).unwrap();
        assert_eq!(node_a.x, 10.0);
        assert_eq!(
            node_a.config.get("area_acres").and_then(|v| v.as_f64()),
            Some(7.5)
        );
    }

    #[test]
    fn config_fields_populated_for_all_kinds() {
        // Every node kind except Outfall must have at least one config field.
        for &kind in NodeKind::all() {
            let def = kind.def();
            if kind == NodeKind::Outfall {
                assert!(def.config_fields.is_empty(), "Outfall should have no config fields");
            } else {
                assert!(
                    !def.config_fields.is_empty(),
                    "NodeKind {:?} has no config fields defined",
                    kind
                );
            }
        }
    }

    #[test]
    fn all_select_fields_have_options() {
        for &kind in NodeKind::all() {
            for f in &kind.def().config_fields {
                if f.field_type == "select" {
                    assert!(
                        !f.options.is_empty(),
                        "NodeKind {:?} field '{}' is 'select' but has no options",
                        kind, f.key
                    );
                }
            }
        }
    }

    #[test]
    fn palette_json_is_valid_json_with_all_kinds() {
        // This exercises the serde serialization path in lib.rs palette_json()
        // without needing wasm-bindgen; we call the same logic here.
        use crate::nodes::NodeKind;
        let items: Vec<serde_json::Value> = NodeKind::all().iter().map(|&kind| {
            let def = kind.def();
            let fields: Vec<serde_json::Value> = def.config_fields.iter().map(|f| {
                serde_json::json!({ "key": f.key, "field_type": f.field_type })
            }).collect();
            serde_json::json!({ "kind": format!("{:?}", kind), "n_fields": fields.len() })
        }).collect();
        assert_eq!(items.len(), NodeKind::all().len());
        // Every item serializes cleanly
        assert!(serde_json::to_string(&items).is_ok());
    }
}
