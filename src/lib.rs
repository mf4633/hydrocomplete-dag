mod chart;
mod dag;
mod interaction;
mod nodes;
mod renderer;
mod templates;

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Window};

use dag::DagModel;
use interaction::{Interaction, InteractionState};
use nodes::NodeKind;
use renderer::Camera;

fn window() -> Window {
    web_sys::window().expect("no window")
}

fn canvas_size(canvas: &HtmlCanvasElement) -> (f64, f64) {
    (canvas.width() as f64, canvas.height() as f64)
}

const HISTORY_LIMIT: usize = 50;

/// The main WASM-exported DAG editor.  One instance per canvas element.
#[wasm_bindgen]
pub struct DagEditor {
    dag: DagModel,
    history: Vec<DagModel>,
    future: Vec<DagModel>,
    camera: Camera,
    interaction: Interaction,
    ctx: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
}

#[wasm_bindgen]
impl DagEditor {
    /// Create a new editor attached to `<canvas id="{canvas_id}">`.
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<DagEditor, JsValue> {
        let doc = window().document().ok_or("no document")?;
        let el = doc
            .get_element_by_id(canvas_id)
            .ok_or("canvas not found")?;
        let canvas = el
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| "not a canvas")?;
        let ctx = canvas
            .get_context("2d")?
            .ok_or("no 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "cast failed")?;

        Ok(DagEditor {
            dag: DagModel::new(),
            history: Vec::new(),
            future: Vec::new(),
            camera: Camera::new(),
            interaction: Interaction::new(),
            ctx,
            canvas,
        })
    }

    // ── Input events ─────────────────────────────────────────────────────

    /// Left-button press.  `button`: 0=left, 1=middle, 2=right.
    pub fn mouse_down(&mut self, sx: f64, sy: f64, button: u16) {
        self.interaction.mouse_down(sx, sy, button, &self.dag, &self.camera);
    }

    pub fn mouse_move(&mut self, sx: f64, sy: f64) {
        self.interaction.mouse_move(sx, sy, &mut self.dag, &mut self.camera);
        self.render();
    }

    pub fn mouse_up(&mut self, sx: f64, sy: f64) {
        // push undo before any structural mutation
        let dag_before = self.dag.clone();
        let mutated = self.interaction.mouse_up(sx, sy, &mut self.dag, &self.camera);
        if mutated { self.push_undo_snapshot(dag_before); }
        self.render();
    }

    pub fn wheel(&mut self, delta_y: f64, sx: f64, sy: f64) {
        self.interaction.wheel(delta_y, sx, sy, &mut self.camera);
        self.render();
    }

    pub fn key_down(&mut self, key: &str) -> bool {
        match key {
            "Delete" | "Backspace" => {
                let snap = self.dag.clone();
                let mutated = self.interaction.delete_selected(&mut self.dag);
                if mutated { self.push_undo_snapshot(snap); self.render(); }
                mutated
            }
            _ => false,
        }
    }

    // ── Undo / Redo ────────────────────────────────────────────────────────────

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.future.push(self.dag.clone());
            self.dag = prev;
            self.interaction = Interaction::new();
            self.render();
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.future.pop() {
            self.history.push(self.dag.clone());
            self.dag = next;
            self.interaction = Interaction::new();
            self.render();
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool { !self.history.is_empty() }
    pub fn can_redo(&self) -> bool { !self.future.is_empty() }

    // ── Templates ──────────────────────────────────────────────────────────────

    /// Load a named template.  Returns true on success.
    pub fn load_template(&mut self, name: &str) -> bool {
        if let Some(t) = templates::get(name) {
            let snap = self.dag.clone();
            self.push_undo_snapshot(snap);
            self.dag = t;
            self.interaction = Interaction::new();
            self.camera = Camera::new();
            self.render();
            true
        } else {
            false
        }
    }

    /// JSON array of available template descriptors for the UI dropdown.
    pub fn template_catalog_json() -> String {
        templates::catalog_json()
    }

    // ── Palette drag ─────────────────────────────────────────────────────

    /// Called when the user starts dragging a node type from the HTML palette.
    /// `kind_str` must match the serde snake_case name of a `NodeKind` variant.
    pub fn palette_drag_start(&mut self, kind_str: &str) {
        if let Ok(kind) = serde_json::from_value::<NodeKind>(
            serde_json::Value::String(kind_str.to_string()),
        ) {
            self.interaction.palette_drag_start(kind);
        }
    }

    /// Called on every mousemove while dragging from the palette.
    pub fn palette_drag_move(&mut self, sx: f64, sy: f64) {
        if let InteractionState::DraggingFromPalette { world_x, world_y, .. } =
            &mut self.interaction.state
        {
            let (wx, wy) = self.camera.to_world(sx, sy);
            *world_x = wx;
            *world_y = wy;
        }
        self.render();
    }

    /// Called when the user releases a palette drag on the canvas.
    pub fn palette_drop(&mut self, sx: f64, sy: f64) {
        let dag_before = self.dag.clone();
        let mutated = self.interaction.mouse_up(sx, sy, &mut self.dag, &self.camera);
        if mutated { self.push_undo_snapshot(dag_before); }
        self.render();
    }

    /// Called when a palette drag is cancelled (dropped outside canvas).
    pub fn palette_cancel(&mut self) {
        self.interaction.state = InteractionState::Idle;
        self.render();
    }

    // ── Serialization ─────────────────────────────────────────────────────

    pub fn to_json(&self) -> String {
        self.dag.to_json()
    }

    /// Returns `true` on success.
    pub fn from_json(&mut self, json: &str) -> bool {
        if let Some(dag) = DagModel::from_json(json) {
            self.dag = dag;
            self.render();
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.dag = DagModel::new();
        self.interaction = Interaction::new();
        self.render();
    }

    /// Returns JSON of the topological execution order (array of node ids).
    pub fn execution_order_json(&self) -> String {
        let order: Vec<u32> = self.dag.topological_order().iter().map(|n| n.0).collect();
        serde_json::to_string(&order).unwrap_or_default()
    }

    /// Returns JSON summary of available node kinds (palette + config schema).
    pub fn palette_json() -> String {
        let palette: Vec<serde_json::Value> = NodeKind::all()
            .iter()
            .map(|&kind| {
                let def = kind.def();
                let fields: Vec<serde_json::Value> = def.config_fields.iter().map(|f| {
                    let opts: Vec<serde_json::Value> = f.options.iter().map(|(v, l)| {
                        serde_json::json!({"value": v, "label": l})
                    }).collect();
                    serde_json::json!({
                        "key": f.key,
                        "label": f.label,
                        "field_type": f.field_type,
                        "default_num": f.default_num,
                        "default_str": f.default_str,
                        "unit": f.unit,
                        "options": opts,
                    })
                }).collect();
                serde_json::json!({
                    "kind": serde_json::to_value(kind).unwrap_or_default(),
                    "label": def.label,
                    "category": def.category.name(),
                    "header_fill": def.category.header_fill(),
                    "body_fill": def.category.body_fill(),
                    "border": def.category.border(),
                    "description": def.description,
                    "n_inputs": def.inputs.len(),
                    "n_outputs": def.outputs.len(),
                    "config_fields": fields,
                })
            })
            .collect();
        serde_json::to_string(&palette).unwrap_or_default()
    }

    // ── Node config read/write ─────────────────────────────────────────────

    /// Returns the selected node's JSON (id + kind + config + outputs), or empty string.
    pub fn get_selected_node_json(&self) -> String {
        use crate::interaction::Selection;
        if let Selection::Node(id) = self.interaction.selection {
            if let Some(node) = self.dag.get_node(id) {
                return serde_json::to_string(node).unwrap_or_default();
            }
        }
        String::new()
    }

    /// Update the config of node `node_id` with a JSON object.  Returns true on success.
    pub fn set_node_config(&mut self, node_id: u32, config_json: &str) -> bool {
        let id = dag::NodeId(node_id);
        let config: std::collections::HashMap<String, serde_json::Value> =
            match serde_json::from_str(config_json) {
                Ok(m) => m,
                Err(_) => return false,
            };
        if let Some(node) = self.dag.get_node_mut(id) {
            node.config = config;
            self.render();
            true
        } else {
            false
        }
    }

    fn push_undo_snapshot(&mut self, snapshot: DagModel) {
        if self.history.len() >= HISTORY_LIMIT {
            self.history.remove(0);
        }
        self.history.push(snapshot);
        self.future.clear();
    }

    /// Returns the u32 ID of the selected node, or u32::MAX if nothing selected.
    pub fn selected_node_id(&self) -> u32 {
        use crate::interaction::Selection;
        if let Selection::Node(id) = self.interaction.selection { id.0 } else { u32::MAX }
    }

    /// Apply engine result JSON (updated DAG with `outputs` filled in) to the model.
    pub fn apply_results_json(&mut self, dag_json: &str) -> bool {
        if let Some(dag) = dag::DagModel::from_json(dag_json) {
            // Copy outputs from result dag into our model (preserves layout + config)
            let mut outputs_map: std::collections::HashMap<u32, std::collections::HashMap<String, serde_json::Value>> =
                std::collections::HashMap::new();
            for n in &dag.nodes {
                outputs_map.insert(n.id.0, n.outputs.clone());
            }
            for node in &mut self.dag.nodes {
                if let Some(outs) = outputs_map.get(&node.id.0) {
                    node.outputs = outs.clone();
                }
            }
            self.render();
            true
        } else {
            false
        }
    }

    // ── Rendering ─────────────────────────────────────────────────────────

    pub fn render(&self) {
        let (w, h) = canvas_size(&self.canvas);
        renderer::render(
            &self.ctx,
            &self.dag,
            &self.interaction.state,
            &self.interaction.selection,
            &self.camera,
            w,
            h,
        );
    }

    /// Notify the editor that the canvas has been resized.
    pub fn resize(&self) {
        self.render();
    }

    // ── Chart rendering ────────────────────────────────────────────────────────

    /// Draw a result chart for node `node_id` onto a separate canvas element.
    /// Call after `apply_results_json()`. Returns false if the node has no outputs.
    pub fn render_chart(&self, node_id: u32, chart_canvas_id: &str) -> bool {
        let id = dag::NodeId(node_id);
        let node = match self.dag.get_node(id) {
            Some(n) => n,
            None => return false,
        };
        if node.outputs.is_empty() { return false; }

        let doc = match window().document() {
            Some(d) => d,
            None => return false,
        };
        let el = match doc.get_element_by_id(chart_canvas_id) {
            Some(e) => e,
            None => return false,
        };
        let canvas = match el.dyn_into::<web_sys::HtmlCanvasElement>() {
            Ok(c) => c,
            Err(_) => return false,
        };
        let ctx = match canvas.get_context("2d")
            .ok().flatten()
            .and_then(|o| o.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
        {
            Some(c) => c,
            None => return false,
        };

        let w = canvas.width() as f64;
        let h = canvas.height() as f64;
        let kind_str = serde_json::to_value(node.kind)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();
        let outputs_val = serde_json::to_value(&node.outputs).unwrap_or(serde_json::Value::Null);

        chart::render_node_chart(&ctx, w, h, &kind_str, &outputs_val);
        true
    }

    // ── Zoom controls ─────────────────────────────────────────────────────

    pub fn zoom_in(&mut self) {
        let (w, h) = canvas_size(&self.canvas);
        self.interaction.wheel(-1.0, w / 2.0, h / 2.0, &mut self.camera);
    }

    pub fn zoom_out(&mut self) {
        let (w, h) = canvas_size(&self.canvas);
        self.interaction.wheel(1.0, w / 2.0, h / 2.0, &mut self.camera);
    }

    pub fn zoom_reset(&mut self) {
        self.camera = Camera::new();
        self.render();
    }
}
