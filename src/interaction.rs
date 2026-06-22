use crate::dag::{DagModel, EdgeId, NodeId};
use crate::nodes::{NodeKind, NODE_H, NODE_W};
use crate::renderer::{hit_edge, hit_input_port, hit_node, hit_output_port, Camera};

// ── Selection ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Selection {
    pub nodes: Vec<NodeId>,
    pub edge: Option<EdgeId>,
}

impl Selection {
    pub fn none() -> Self { Self::default() }
    pub fn of_node(id: NodeId) -> Self { Selection { nodes: vec![id], edge: None } }
    pub fn of_edge(id: EdgeId) -> Self { Selection { nodes: Vec::new(), edge: Some(id) } }
    pub fn has_node(&self, id: NodeId) -> bool { self.nodes.contains(&id) }
    pub fn is_empty(&self) -> bool { self.nodes.is_empty() && self.edge.is_none() }
    pub fn primary_node(&self) -> Option<NodeId> { self.nodes.first().copied() }
    pub fn node_count(&self) -> usize { self.nodes.len() }

    fn add(&mut self, id: NodeId) {
        if !self.nodes.contains(&id) { self.nodes.push(id); }
        self.edge = None;
    }
    fn toggle(&mut self, id: NodeId) {
        if let Some(pos) = self.nodes.iter().position(|&n| n == id) {
            self.nodes.remove(pos);
        } else {
            self.nodes.push(id);
        }
        self.edge = None;
    }
}

// ── Interaction state machine ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum InteractionState {
    Idle,
    /// Dragging one or more selected nodes; `offsets` = (nodeId, wx-offset, wy-offset).
    DraggingNodes {
        offsets: Vec<(NodeId, f64, f64)>,
    },
    DraggingFromPalette {
        kind: NodeKind,
        world_x: f64,
        world_y: f64,
    },
    DrawingEdge {
        from_node: NodeId,
        from_port: usize,
        world_x: f64,
        world_y: f64,
    },
    /// Rubber-band area selection.
    RubberBand {
        start_wx: f64,
        start_wy: f64,
        cur_wx: f64,
        cur_wy: f64,
    },
    Panning {
        start_sx: f64,
        start_sy: f64,
        start_pan_x: f64,
        start_pan_y: f64,
    },
}

pub struct Interaction {
    pub state: InteractionState,
    pub selection: Selection,
}

impl Interaction {
    pub fn new() -> Self {
        Interaction { state: InteractionState::Idle, selection: Selection::none() }
    }

    // ── Mouse events ──────────────────────────────────────────────────────────

    /// `shift` = Shift key held (additive selection).
    pub fn mouse_down(
        &mut self,
        sx: f64, sy: f64,
        button: u16,
        shift: bool,
        dag: &DagModel,
        camera: &Camera,
    ) {
        let (wx, wy) = camera.to_world(sx, sy);

        // Middle / right → pan
        if button == 1 || button == 2 {
            self.state = InteractionState::Panning {
                start_sx: sx, start_sy: sy,
                start_pan_x: camera.pan_x, start_pan_y: camera.pan_y,
            };
            return;
        }

        // Output port → begin edge draw
        if let Some((node_id, port_idx)) = hit_output_port(dag, wx, wy) {
            self.state = InteractionState::DrawingEdge {
                from_node: node_id, from_port: port_idx,
                world_x: wx, world_y: wy,
            };
            return;
        }

        // Node body
        if let Some(node_id) = hit_node(dag, wx, wy) {
            if shift {
                self.selection.toggle(node_id);
            } else if !self.selection.has_node(node_id) {
                // Replace selection
                self.selection = Selection::of_node(node_id);
            }
            // Build offsets for ALL selected nodes so group drag works
            let offsets: Vec<(NodeId, f64, f64)> = self.selection.nodes.iter()
                .filter_map(|&nid| dag.get_node(nid).map(|n| (nid, wx - n.x, wy - n.y)))
                .collect();
            self.state = InteractionState::DraggingNodes { offsets };
            return;
        }

        // Edge
        if let Some(edge_id) = hit_edge(dag, wx, wy) {
            if shift {
                self.selection.edge = Some(edge_id);
            } else {
                self.selection = Selection::of_edge(edge_id);
            }
            self.state = InteractionState::Idle;
            return;
        }

        // Empty canvas → rubber-band (additive if shift, replace otherwise)
        if !shift { self.selection = Selection::none(); }
        self.state = InteractionState::RubberBand {
            start_wx: wx, start_wy: wy, cur_wx: wx, cur_wy: wy,
        };
    }

    pub fn mouse_move(
        &mut self,
        sx: f64, sy: f64,
        dag: &mut DagModel,
        camera: &mut Camera,
        snap: Option<f64>,
    ) {
        let (wx, wy) = camera.to_world(sx, sy);

        match &mut self.state {
            InteractionState::DraggingNodes { offsets } => {
                for &(nid, ox, oy) in offsets.iter() {
                    if let Some(node) = dag.get_node_mut(nid) {
                        let mut nx = wx - ox;
                        let mut ny = wy - oy;
                        if let Some(g) = snap {
                            nx = (nx / g).round() * g;
                            ny = (ny / g).round() * g;
                        }
                        node.x = nx;
                        node.y = ny;
                    }
                }
            }
            InteractionState::DrawingEdge { world_x, world_y, .. } => {
                *world_x = wx; *world_y = wy;
            }
            InteractionState::DraggingFromPalette { world_x, world_y, .. } => {
                *world_x = wx; *world_y = wy;
            }
            InteractionState::RubberBand { cur_wx, cur_wy, .. } => {
                *cur_wx = wx; *cur_wy = wy;
            }
            InteractionState::Panning { start_sx, start_sy, start_pan_x, start_pan_y } => {
                let (ssx, ssy, spx, spy) = (*start_sx, *start_sy, *start_pan_x, *start_pan_y);
                camera.pan_x = spx + (sx - ssx);
                camera.pan_y = spy + (sy - ssy);
            }
            InteractionState::Idle => {}
        }
    }

    /// Returns true if the DAG was structurally mutated (not just a position move).
    pub fn mouse_up(
        &mut self,
        sx: f64, sy: f64,
        dag: &mut DagModel,
        camera: &Camera,
    ) -> bool {
        let (wx, wy) = camera.to_world(sx, sy);
        let mut mutated = false;
        let prev = std::mem::replace(&mut self.state, InteractionState::Idle);

        match prev {
            InteractionState::DrawingEdge { from_node, from_port, .. } => {
                if let Some((to_node, to_port)) = hit_input_port(dag, wx, wy) {
                    if dag.add_edge(from_node, from_port, to_node, to_port).is_some() {
                        mutated = true;
                    }
                }
            }
            InteractionState::DraggingFromPalette { kind, world_x, world_y } => {
                let x = world_x - NODE_W / 2.0;
                let y = world_y - NODE_H / 2.0;
                let id = dag.add_node(kind, x, y);
                self.selection = Selection::of_node(id);
                mutated = true;
            }
            InteractionState::RubberBand { start_wx, start_wy, cur_wx, cur_wy } => {
                let w = (cur_wx - start_wx).abs();
                let h = (cur_wy - start_wy).abs();
                if w > 5.0 || h > 5.0 {
                    let rx = start_wx.min(cur_wx);
                    let ry = start_wy.min(cur_wy);
                    let rr = rx + w;
                    let rb = ry + h;
                    let in_band: Vec<NodeId> = dag.nodes.iter()
                        .filter(|n| n.x < rr && n.x + NODE_W > rx && n.y < rb && n.y + NODE_H > ry)
                        .map(|n| n.id)
                        .collect();
                    for nid in in_band { self.selection.add(nid); }
                }
            }
            InteractionState::DraggingNodes { .. } => {
                mutated = true; // positions changed — trigger undo snapshot
            }
            _ => {}
        }

        mutated
    }

    pub fn wheel(&mut self, delta_y: f64, sx: f64, sy: f64, camera: &mut Camera) {
        let f = if delta_y < 0.0 { 1.1 } else { 1.0 / 1.1 };
        let z = (camera.zoom * f).clamp(0.12, 4.0);
        camera.pan_x = sx - (sx - camera.pan_x) * (z / camera.zoom);
        camera.pan_y = sy - (sy - camera.pan_y) * (z / camera.zoom);
        camera.zoom = z;
    }

    /// Delete all selected nodes and edges. Returns true if anything was removed.
    pub fn delete_selected(&mut self, dag: &mut DagModel) -> bool {
        let mut mutated = false;
        if let Some(eid) = self.selection.edge.take() {
            dag.remove_edge(eid);
            mutated = true;
        }
        for nid in self.selection.nodes.drain(..) {
            dag.remove_node(nid);
            mutated = true;
        }
        self.selection = Selection::none();
        mutated
    }

    pub fn palette_drag_start(&mut self, kind: NodeKind) {
        self.state = InteractionState::DraggingFromPalette {
            kind, world_x: 0.0, world_y: 0.0,
        };
    }

    pub fn select_all(&mut self, dag: &DagModel) {
        self.selection.nodes = dag.nodes.iter().map(|n| n.id).collect();
        self.selection.edge = None;
    }
}
