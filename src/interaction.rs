use crate::dag::{DagModel, EdgeId, NodeId};
use crate::nodes::NodeKind;
use crate::renderer::{hit_edge, hit_input_port, hit_node, hit_output_port, Camera};

#[derive(Debug, Clone)]
pub enum InteractionState {
    Idle,
    DraggingNode {
        node_id: NodeId,
        offset_x: f64,
        offset_y: f64,
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
    Panning {
        start_sx: f64,
        start_sy: f64,
        start_pan_x: f64,
        start_pan_y: f64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    None,
    Node(NodeId),
    Edge(EdgeId),
}

pub struct Interaction {
    pub state: InteractionState,
    pub selection: Selection,
}

impl Interaction {
    pub fn new() -> Self {
        Interaction {
            state: InteractionState::Idle,
            selection: Selection::None,
        }
    }

    /// Mouse button 1 pressed (primary). `sx/sy` = screen coords.
    pub fn mouse_down(
        &mut self,
        sx: f64,
        sy: f64,
        button: u16,
        dag: &DagModel,
        camera: &Camera,
    ) {
        let (wx, wy) = camera.to_world(sx, sy);

        // Middle mouse or right mouse → pan
        if button == 1 || button == 2 {
            self.state = InteractionState::Panning {
                start_sx: sx,
                start_sy: sy,
                start_pan_x: camera.pan_x,
                start_pan_y: camera.pan_y,
            };
            return;
        }

        // Check output port first (starts edge drawing)
        if let Some((node_id, port_idx)) = hit_output_port(dag, wx, wy) {
            self.state = InteractionState::DrawingEdge {
                from_node: node_id,
                from_port: port_idx,
                world_x: wx,
                world_y: wy,
            };
            return;
        }

        // Check node body
        if let Some(node_id) = hit_node(dag, wx, wy) {
            self.selection = Selection::Node(node_id);
            let node = dag.get_node(node_id).unwrap();
            self.state = InteractionState::DraggingNode {
                node_id,
                offset_x: wx - node.x,
                offset_y: wy - node.y,
            };
            return;
        }

        // Check edge
        if let Some(edge_id) = hit_edge(dag, wx, wy) {
            self.selection = Selection::Edge(edge_id);
            self.state = InteractionState::Idle;
            return;
        }

        // Click empty canvas → deselect
        self.selection = Selection::None;
        self.state = InteractionState::Idle;
    }

    /// Mouse moved (with or without button held).
    pub fn mouse_move(
        &mut self,
        sx: f64,
        sy: f64,
        dag: &mut DagModel,
        camera: &mut Camera,
    ) {
        let (wx, wy) = camera.to_world(sx, sy);

        match &mut self.state {
            InteractionState::DraggingNode { node_id, offset_x, offset_y } => {
                let nx = *node_id;
                let ox = *offset_x;
                let oy = *offset_y;
                if let Some(node) = dag.get_node_mut(nx) {
                    node.x = wx - ox;
                    node.y = wy - oy;
                }
            }
            InteractionState::DrawingEdge { world_x, world_y, .. } => {
                *world_x = wx;
                *world_y = wy;
            }
            InteractionState::DraggingFromPalette { world_x, world_y, .. } => {
                *world_x = wx;
                *world_y = wy;
            }
            InteractionState::Panning { start_sx, start_sy, start_pan_x, start_pan_y } => {
                let ssx = *start_sx;
                let ssy = *start_sy;
                let spx = *start_pan_x;
                let spy = *start_pan_y;
                camera.pan_x = spx + (sx - ssx);
                camera.pan_y = spy + (sy - ssy);
            }
            InteractionState::Idle => {}
        }
    }

    /// Mouse button released. Returns `true` if DAG was mutated.
    pub fn mouse_up(
        &mut self,
        sx: f64,
        sy: f64,
        dag: &mut DagModel,
        camera: &Camera,
    ) -> bool {
        let (wx, wy) = camera.to_world(sx, sy);
        let mut mutated = false;

        let prev = self.state.clone();
        self.state = InteractionState::Idle;

        match prev {
            InteractionState::DrawingEdge { from_node, from_port, .. } => {
                if let Some((to_node, to_port)) = hit_input_port(dag, wx, wy) {
                    if dag.add_edge(from_node, from_port, to_node, to_port).is_some() {
                        mutated = true;
                    }
                }
            }
            InteractionState::DraggingFromPalette { kind, world_x, world_y } => {
                let x = world_x - crate::nodes::NODE_W / 2.0;
                let y = world_y - crate::nodes::NODE_H / 2.0;
                let id = dag.add_node(kind, x, y);
                self.selection = Selection::Node(id);
                mutated = true;
            }
            _ => {}
        }

        mutated
    }

    /// Scroll wheel zoom, centered on (sx, sy).
    pub fn wheel(&mut self, delta_y: f64, sx: f64, sy: f64, camera: &mut Camera) {
        let factor = if delta_y < 0.0 { 1.1 } else { 1.0 / 1.1 };
        let new_zoom = (camera.zoom * factor).clamp(0.2, 4.0);
        // Zoom toward cursor
        camera.pan_x = sx - (sx - camera.pan_x) * (new_zoom / camera.zoom);
        camera.pan_y = sy - (sy - camera.pan_y) * (new_zoom / camera.zoom);
        camera.zoom = new_zoom;
    }

    /// Delete selected element.  Returns true if mutated.
    pub fn delete_selected(&mut self, dag: &mut DagModel) -> bool {
        match self.selection.clone() {
            Selection::Node(id) => {
                dag.remove_node(id);
                self.selection = Selection::None;
                true
            }
            Selection::Edge(id) => {
                dag.remove_edge(id);
                self.selection = Selection::None;
                true
            }
            Selection::None => false,
        }
    }

    /// Begin a palette drag (called from JS palette mousedown handler).
    pub fn palette_drag_start(&mut self, kind: NodeKind) {
        self.state = InteractionState::DraggingFromPalette {
            kind,
            world_x: 0.0,
            world_y: 0.0,
        };
    }
}
