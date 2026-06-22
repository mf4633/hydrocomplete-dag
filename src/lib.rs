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
    clipboard: Option<(String, String)>,  // (kind_json, config_json)
    snap_grid: Option<f64>,               // None = off, Some(g) = snap to g px
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
            clipboard: None,
            snap_grid: None,
            camera: Camera::new(),
            interaction: Interaction::new(),
            ctx,
            canvas,
        })
    }

    // ── Input events ─────────────────────────────────────────────────────

    /// Left-button press.  `button`: 0=left, 1=middle, 2=right.  `shift`: Shift key held.
    pub fn mouse_down(&mut self, sx: f64, sy: f64, button: u16, shift: bool) {
        self.interaction.mouse_down(sx, sy, button, shift, &self.dag, &self.camera);
    }

    pub fn mouse_move(&mut self, sx: f64, sy: f64) {
        self.interaction.mouse_move(sx, sy, &mut self.dag, &mut self.camera, self.snap_grid);
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

    /// `key` is the bare key string; `ctrl` true when Ctrl/Meta held.
    pub fn key_down(&mut self, key: &str) -> bool {
        match key {
            "Delete" | "Backspace" => {
                let snap = self.dag.clone();
                let mutated = self.interaction.delete_selected(&mut self.dag);
                if mutated { self.push_undo_snapshot(snap); self.render(); }
                mutated
            }
            "c" => { self.copy_selected(); false }
            "v" => { self.paste() }
            "d" => { self.copy_selected(); self.paste() }
            "a" => { self.select_all_nodes(); false }   // Ctrl+A = select all
            "f" => { self.zoom_fit(); false }            // Ctrl+F = fit (was Ctrl+A)
            "l" => { self.layout(); false }
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

    // ── Selection utilities ────────────────────────────────────────────────────

    pub fn select_all_nodes(&mut self) {
        self.interaction.select_all(&self.dag);
        self.render();
    }

    pub fn selected_node_count(&self) -> usize {
        self.interaction.selection.node_count()
    }

    // ── Snap to grid ──────────────────────────────────────────────────────────

    /// Toggle snap-to-grid.  `grid_size` = world-space grid size in pixels (e.g. 20).
    pub fn toggle_snap(&mut self, grid_size: f64) -> bool {
        if self.snap_grid.is_some() {
            self.snap_grid = None;
            false
        } else {
            self.snap_grid = Some(grid_size);
            true
        }
    }

    pub fn snap_enabled(&self) -> bool { self.snap_grid.is_some() }

    // ── Info ──────────────────────────────────────────────────────────────────

    pub fn zoom_percent(&self) -> u32 { (self.camera.zoom * 100.0).round() as u32 }
    pub fn node_count(&self) -> usize { self.dag.nodes.len() }
    pub fn edge_count(&self) -> usize { self.dag.edges.len() }

    // ── Layout & view ─────────────────────────────────────────────────────────

    /// Fit all nodes into view with padding.
    pub fn zoom_fit(&mut self) {
        use crate::nodes::{NODE_H, NODE_W};
        if self.dag.nodes.is_empty() { self.camera = Camera::new(); self.render(); return; }
        let (nx, ny, xx, xy) = self.dag.nodes.iter().fold(
            (f64::MAX, f64::MAX, f64::MIN_POSITIVE, f64::MIN_POSITIVE),
            |(a,b,c,d), n| (a.min(n.x), b.min(n.y), c.max(n.x+NODE_W), d.max(n.y+NODE_H))
        );
        let (cw, ch) = canvas_size(&self.canvas);
        let pad = 60.0;
        let span_x = (xx - nx).max(1.0);
        let span_y = (xy - ny).max(1.0);
        let zoom = ((cw - pad*2.0)/span_x).min((ch - pad*2.0)/span_y).min(1.5).max(0.15);
        self.camera.zoom  = zoom;
        self.camera.pan_x = (cw - span_x * zoom) / 2.0 - nx * zoom;
        self.camera.pan_y = (ch - span_y * zoom) / 2.0 - ny * zoom;
        self.render();
    }

    /// Hierarchical auto-layout: left-to-right layering with vertical centering.
    pub fn layout(&mut self) {
        use std::collections::HashMap;
        use crate::dag::NodeId;

        if self.dag.nodes.is_empty() { return; }
        let snap = self.dag.clone();
        let topo = self.dag.topological_order();

        // Assign layer = longest path from any source
        let mut layer: HashMap<u32, usize> = HashMap::new();
        for &nid in &topo {
            let max_pred = self.dag.edges.iter()
                .filter(|e| e.to_node == nid)
                .filter_map(|e| layer.get(&e.from_node.0))
                .max()
                .map(|&l| l + 1)
                .unwrap_or(0);
            layer.insert(nid.0, max_pred);
        }

        let max_layer = layer.values().max().copied().unwrap_or(0);
        let mut layers: Vec<Vec<NodeId>> = vec![Vec::new(); max_layer + 1];
        for &nid in &topo {
            layers[*layer.get(&nid.0).unwrap_or(&0)].push(nid);
        }

        // Within each layer, sort by average y of predecessors (reduce crossings)
        let node_center_y: HashMap<u32, f64> = self.dag.nodes.iter()
            .map(|n| (n.id.0, n.y)).collect();

        const DX: f64 = 220.0;
        const DY: f64 = 120.0;
        const PAD_X: f64 = 40.0;
        const PAD_Y: f64 = 60.0;

        for (l, layer_nodes) in layers.iter_mut().enumerate() {
            // Sort by avg predecessor y to minimise crossings
            layer_nodes.sort_by(|&a, &b| {
                let avg_y = |nid: dag::NodeId| -> f64 {
                    let preds: Vec<f64> = self.dag.edges.iter()
                        .filter(|e| e.to_node == nid)
                        .filter_map(|e| node_center_y.get(&e.from_node.0))
                        .cloned().collect();
                    if preds.is_empty() { 0.0 } else { preds.iter().sum::<f64>() / preds.len() as f64 }
                };
                avg_y(a).partial_cmp(&avg_y(b)).unwrap_or(std::cmp::Ordering::Equal)
            });

            let n = layer_nodes.len() as f64;
            let total_h = n * DY;
            for (i, &nid) in layer_nodes.iter().enumerate() {
                if let Some(node) = self.dag.get_node_mut(nid) {
                    node.x = PAD_X + l as f64 * DX;
                    node.y = PAD_Y + i as f64 * DY - total_h / 2.0 + DY / 2.0;
                    let _ = node;
                }
            }
        }

        self.push_undo_snapshot(snap);
        self.zoom_fit();
    }

    // ── Copy / Paste ────────────────────────────────────────────────────────────

    /// Copy the selected node's kind + config into the in-memory clipboard.
    pub fn copy_selected(&mut self) -> bool {
        if let Some(id) = self.interaction.selection.primary_node() {
            if let Some(node) = self.dag.get_node(id) {
                let kind_json = serde_json::to_string(&node.kind).unwrap_or_default();
                let cfg_json  = serde_json::to_string(&node.config).unwrap_or_default();
                self.clipboard = Some((kind_json, cfg_json));
                return true;
            }
        }
        false
    }

    pub fn paste(&mut self) -> bool {
        let (kind_json, cfg_json) = match &self.clipboard {
            Some(c) => (c.0.clone(), c.1.clone()),
            None => return false,
        };
        let kind: NodeKind = match serde_json::from_str(&kind_json) {
            Ok(k) => k,
            Err(_) => return false,
        };
        let config: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_str(&cfg_json).unwrap_or_default();

        let (px, py) = if let Some(id) = self.interaction.selection.primary_node() {
            if let Some(n) = self.dag.get_node(id) { (n.x + 30.0, n.y + 30.0) }
            else { (200.0, 200.0) }
        } else {
            let (cw, ch) = canvas_size(&self.canvas);
            self.camera.to_world(cw / 2.0, ch / 2.0)
        };

        let snap = self.dag.clone();
        let new_id = self.dag.add_node(kind, px, py);
        if let Some(node) = self.dag.get_node_mut(new_id) {
            node.config = config;
        }
        self.interaction.selection = crate::interaction::Selection::of_node(new_id);
        self.push_undo_snapshot(snap);
        self.render();
        true
    }

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
        if let Some(id) = self.interaction.selection.primary_node() {
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
        self.interaction.selection.primary_node().map(|n| n.0).unwrap_or(u32::MAX)
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

    // ── Minimap ────────────────────────────────────────────────────────────────

    /// Render a scaled-down overview of the DAG onto a separate minimap canvas.
    /// Draws node rects, edges, and a viewport indicator box.
    pub fn render_minimap(&self, minimap_id: &str, main_canvas_w: f64, main_canvas_h: f64) -> bool {
        use crate::nodes::{NODE_H, NODE_W};

        let doc = match window().document() { Some(d) => d, None => return false };
        let el  = match doc.get_element_by_id(minimap_id) { Some(e) => e, None => return false };
        let mc  = match el.dyn_into::<web_sys::HtmlCanvasElement>() { Ok(c) => c, Err(_) => return false };
        let ctx = match mc.get_context("2d").ok().flatten()
            .and_then(|o| o.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
        { Some(c) => c, None => return false };

        let mw = mc.width()  as f64;
        let mh = mc.height() as f64;

        // World bounds
        if self.dag.nodes.is_empty() { return true; }
        let (wnx, wny, wxx, wxy) = self.dag.nodes.iter().fold(
            (f64::MAX, f64::MAX, f64::MIN_POSITIVE, f64::MIN_POSITIVE),
            |(nx, ny, xx, xy), n| (nx.min(n.x), ny.min(n.y), xx.max(n.x + NODE_W), xy.max(n.y + NODE_H))
        );
        let pad = 20.0;
        let wspan_x = (wxx - wnx + pad * 2.0).max(1.0);
        let wspan_y = (wxy - wny + pad * 2.0).max(1.0);
        let scale = (mw / wspan_x).min(mh / wspan_y);

        let wx = |x: f64| (x - wnx + pad) * scale;
        let wy = |y: f64| (y - wny + pad) * scale;

        // Background
        ctx.set_fill_style(&JsValue::from_str("#161b22"));
        ctx.fill_rect(0.0, 0.0, mw, mh);

        // Edges
        ctx.set_stroke_style(&JsValue::from_str("#30363d"));
        ctx.set_line_width(0.8);
        for edge in &self.dag.edges {
            if let (Some(fn_), Some(tn)) = (self.dag.get_node(edge.from_node), self.dag.get_node(edge.to_node)) {
                let x1 = wx(fn_.x + NODE_W);
                let y1 = wy(fn_.y + NODE_H / 2.0);
                let x2 = wx(tn.x);
                let y2 = wy(tn.y + NODE_H / 2.0);
                ctx.begin_path();
                ctx.move_to(x1, y1);
                let dx = ((x2 - x1).abs() * 0.5).max(20.0 * scale);
                ctx.bezier_curve_to(x1 + dx, y1, x2 - dx, y2, x2, y2);
                ctx.stroke();
            }
        }

        // Nodes
        for node in &self.dag.nodes {
            let def = node.kind.def();
            let fill = def.category.header_fill();
            let nx = wx(node.x);
            let ny = wy(node.y);
            let nw = NODE_W * scale;
            let nh = NODE_H * scale;
            ctx.set_fill_style(&JsValue::from_str(fill));
            ctx.fill_rect(nx, ny, nw, nh);
        }

        // Viewport indicator (what the main canvas currently shows)
        let vp_wx = (-self.camera.pan_x) / self.camera.zoom;
        let vp_wy = (-self.camera.pan_y) / self.camera.zoom;
        let vp_ww = main_canvas_w / self.camera.zoom;
        let vp_wh = main_canvas_h / self.camera.zoom;

        let rx = wx(vp_wx);
        let ry = wy(vp_wy);
        let rw = vp_ww * scale;
        let rh = vp_wh * scale;

        ctx.set_stroke_style(&JsValue::from_str("#58a6ff"));
        ctx.set_line_width(1.5);
        ctx.stroke_rect(rx, ry, rw, rh);
        ctx.set_fill_style(&JsValue::from_str("rgba(88,166,255,0.08)"));
        ctx.fill_rect(rx, ry, rw, rh);

        true
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
