use std::f64::consts::PI;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d as Ctx;

use crate::dag::{DagModel, EdgeId, NodeId};
use crate::interaction::{InteractionState, Selection};
use crate::nodes::{abs_port_pos, NodeKind, HEADER_H, NODE_H, NODE_W, PORT_R};

const GRID_SIZE: f64 = 40.0;
const CORNER_R: f64 = 8.0;
const ARROW_LEN: f64 = 12.0;
const ARROW_HALF: f64 = 5.0;

pub struct Camera {
    pub pan_x: f64,
    pub pan_y: f64,
    pub zoom: f64,
}

impl Camera {
    pub fn new() -> Self {
        Camera { pan_x: 220.0, pan_y: 40.0, zoom: 1.0 }
    }

    pub fn to_world(&self, sx: f64, sy: f64) -> (f64, f64) {
        ((sx - self.pan_x) / self.zoom, (sy - self.pan_y) / self.zoom)
    }

    pub fn apply(&self, ctx: &Ctx, canvas_w: f64, canvas_h: f64) {
        let _ = ctx.set_transform(self.zoom, 0.0, 0.0, self.zoom, self.pan_x, self.pan_y);
        // clip to canvas (in world space)
        let w = canvas_w / self.zoom;
        let h = canvas_h / self.zoom;
        ctx.begin_path();
        ctx.rect((-self.pan_x) / self.zoom, (-self.pan_y) / self.zoom, w, h);
        ctx.clip();
    }
}

pub fn render(
    ctx: &Ctx,
    dag: &DagModel,
    state: &InteractionState,
    selection: &Selection,
    camera: &Camera,
    canvas_w: f64,
    canvas_h: f64,
) {
    // Reset transform to screen space for full clear
    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    ctx.set_fill_style(&JsValue::from_str("#f8f9fa"));
    ctx.fill_rect(0.0, 0.0, canvas_w, canvas_h);

    // Apply camera transform for world-space drawing
    ctx.save();
    camera.apply(ctx, canvas_w, canvas_h);

    draw_grid(ctx, camera, canvas_w, canvas_h);

    // Draw edges (behind nodes)
    for edge in &dag.edges {
        let selected = selection.edge == Some(edge.id);
        if let (Some(fn_), Some(tn)) = (dag.get_node(edge.from_node), dag.get_node(edge.to_node)) {
            let (x1, y1) = abs_port_pos(fn_.kind, fn_.x, fn_.y, edge.from_port, true);
            let (x2, y2) = abs_port_pos(tn.kind, tn.x, tn.y, edge.to_port, false);
            draw_edge(ctx, x1, y1, x2, y2, selected, false);
        }
    }

    // Draw edge preview while drawing
    if let InteractionState::DrawingEdge { from_node, from_port, world_x, world_y } = state {
        if let Some(fn_) = dag.get_node(*from_node) {
            let (x1, y1) = abs_port_pos(fn_.kind, fn_.x, fn_.y, *from_port, true);
            draw_edge(ctx, x1, y1, *world_x, *world_y, false, true);
        }
    }

    // When drawing an edge, highlight all input ports as drop targets
    let highlighting_ports = matches!(state, InteractionState::DrawingEdge { .. });

    // Draw nodes (on top of edges)
    for node in &dag.nodes {
        let selected = selection.has_node(node.id);
        let dragging = matches!(state, InteractionState::DraggingNodes { .. }) && selected;
        draw_node(ctx, dag, node.id, selected, dragging);

        if highlighting_ports {
            let def = node.kind.def();
            for i in 0..def.inputs.len() {
                let (px, py) = abs_port_pos(node.kind, node.x, node.y, i, false);
                ctx.begin_path();
                let _ = ctx.arc(px, py, PORT_R + 4.0, 0.0, 2.0 * std::f64::consts::PI);
                ctx.set_stroke_style(&JsValue::from_str("#58a6ff"));
                ctx.set_line_width(2.0);
                let dash = js_sys::Array::new();
                let _ = ctx.set_line_dash(&dash);
                ctx.stroke();
            }
        }
    }

    // Rubber-band selection rect
    if let InteractionState::RubberBand { start_wx, start_wy, cur_wx, cur_wy } = state {
        let (swx, swy, cwx, cwy) = (*start_wx, *start_wy, *cur_wx, *cur_wy);
        let w = (cwx - swx).abs();
        let h = (cwy - swy).abs();
        if w > 2.0 || h > 2.0 {
            let rx = swx.min(cwx);
            let ry = swy.min(cwy);
            ctx.set_fill_style(&JsValue::from_str("rgba(88,166,255,0.08)"));
            ctx.fill_rect(rx, ry, w, h);
            let dash = js_sys::Array::new();
            dash.push(&JsValue::from_f64(5.0));
            dash.push(&JsValue::from_f64(3.0));
            let _ = ctx.set_line_dash(&dash);
            ctx.set_stroke_style(&JsValue::from_str("#58a6ff"));
            ctx.set_line_width(1.0);
            ctx.stroke_rect(rx, ry, w, h);
            let _ = ctx.set_line_dash(&js_sys::Array::new());
        }
    }

    // Ghost node when dragging from palette
    if let InteractionState::DraggingFromPalette { kind, world_x, world_y } = state {
        draw_ghost_node(ctx, *kind, *world_x, *world_y);
    }

    ctx.restore();

    // Draw palette rail in screen space (no camera transform)
    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
}

fn draw_grid(ctx: &Ctx, camera: &Camera, canvas_w: f64, canvas_h: f64) {
    ctx.set_global_alpha(0.35);
    ctx.set_fill_style(&JsValue::from_str("#9e9e9e"));

    let world_left = (-camera.pan_x) / camera.zoom;
    let world_top = (-camera.pan_y) / camera.zoom;
    let world_right = world_left + canvas_w / camera.zoom;
    let world_bottom = world_top + canvas_h / camera.zoom;

    let start_x = (world_left / GRID_SIZE).floor() * GRID_SIZE;
    let start_y = (world_top / GRID_SIZE).floor() * GRID_SIZE;

    let mut gx = start_x;
    while gx <= world_right {
        let mut gy = start_y;
        while gy <= world_bottom {
            ctx.fill_rect(gx - 1.0, gy - 1.0, 2.0, 2.0);
            gy += GRID_SIZE;
        }
        gx += GRID_SIZE;
    }
    ctx.set_global_alpha(1.0);
}

fn draw_edge(ctx: &Ctx, x1: f64, y1: f64, x2: f64, y2: f64, selected: bool, preview: bool) {
    let dx = (x2 - x1).abs().max(60.0) * 0.55;

    ctx.begin_path();
    ctx.move_to(x1, y1);
    ctx.bezier_curve_to(x1 + dx, y1, x2 - dx, y2, x2, y2);

    if preview {
        let dash = js_sys::Array::new();
        dash.push(&JsValue::from_f64(6.0));
        dash.push(&JsValue::from_f64(4.0));
        let _ = ctx.set_line_dash(&dash);
        ctx.set_stroke_style(&JsValue::from_str("#90a4ae"));
        ctx.set_line_width(1.5);
    } else if selected {
        let dash = js_sys::Array::new();
        let _ = ctx.set_line_dash(&dash);
        ctx.set_stroke_style(&JsValue::from_str("#ff6f00"));
        ctx.set_line_width(2.5);
    } else {
        let dash = js_sys::Array::new();
        let _ = ctx.set_line_dash(&dash);
        ctx.set_stroke_style(&JsValue::from_str("#455a64"));
        ctx.set_line_width(1.8);
    }
    ctx.stroke();

    // Arrowhead: tangent at endpoint is always from (x2-dx, y2) → (x2, y2),
    // i.e. horizontal pointing right (angle = 0), since both control points
    // share the same y-coordinate as their respective endpoints.
    if !preview {
        draw_arrowhead(ctx, x2, y2, 0.0_f64.atan2(dx.max(1.0)), selected);
    }

    // Reset line dash
    let _ = ctx.set_line_dash(&js_sys::Array::new());
}

fn draw_arrowhead(ctx: &Ctx, tip_x: f64, tip_y: f64, angle: f64, selected: bool) {
    let color = if selected { "#ff6f00" } else { "#455a64" };
    let base_x = tip_x - ARROW_LEN * angle.cos();
    let base_y = tip_y - ARROW_LEN * angle.sin();
    let perp = angle + PI / 2.0;
    ctx.begin_path();
    ctx.move_to(tip_x, tip_y);
    ctx.line_to(base_x + ARROW_HALF * perp.cos(), base_y + ARROW_HALF * perp.sin());
    ctx.line_to(base_x - ARROW_HALF * perp.cos(), base_y - ARROW_HALF * perp.sin());
    ctx.close_path();
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.fill();
}

pub fn draw_node(ctx: &Ctx, dag: &DagModel, node_id: NodeId, selected: bool, dragging: bool) {
    let node = match dag.get_node(node_id) {
        Some(n) => n,
        None => return,
    };
    let def = node.kind.def();
    let x = node.x;
    let y = node.y;

    ctx.save();
    if dragging {
        ctx.set_global_alpha(0.75);
    }

    // Drop shadow
    ctx.set_shadow_blur(if selected { 10.0 } else { 4.0 });
    ctx.set_shadow_color(if selected { "rgba(255,111,0,0.5)" } else { "rgba(0,0,0,0.18)" });
    ctx.set_shadow_offset_x(2.0);
    ctx.set_shadow_offset_y(2.0);

    // Body
    ctx.set_fill_style(&JsValue::from_str(def.category.body_fill()));
    ctx.set_stroke_style(&JsValue::from_str(if selected { "#ff6f00" } else { def.category.border() }));
    ctx.set_line_width(if selected { 2.5 } else { 1.5 });
    rounded_rect_path(ctx, x, y, NODE_W, NODE_H, CORNER_R);
    ctx.fill();
    ctx.stroke();

    // Disable shadow for inner elements
    ctx.set_shadow_blur(0.0);
    ctx.set_shadow_offset_x(0.0);
    ctx.set_shadow_offset_y(0.0);

    // Header
    ctx.set_fill_style(&JsValue::from_str(def.category.header_fill()));
    rounded_rect_top_path(ctx, x, y, NODE_W, HEADER_H, CORNER_R);
    ctx.fill();

    // Header label
    ctx.set_fill_style(&JsValue::from_str("#ffffff"));
    ctx.set_font("bold 12px 'Segoe UI', Arial, sans-serif");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(def.label, x + NODE_W / 2.0, y + HEADER_H / 2.0);

    // Body: description or output value (if computed)
    let body_cx = x + NODE_W / 2.0;
    let body_cy = y + HEADER_H + (NODE_H - HEADER_H) / 2.0;

    if let Some(err) = node.outputs.get("error") {
        ctx.set_fill_style(&JsValue::from_str("#c62828"));
        ctx.set_font("bold 9px 'Segoe UI', Arial, sans-serif");
        let msg = err.as_str().unwrap_or("error").chars().take(22).collect::<String>();
        let _ = ctx.fill_text(&format!("⚠ {}", msg), body_cx, body_cy);
    } else if let Some(v0) = node.outputs.get("0") {
        // Show primary output value
        ctx.set_fill_style(&JsValue::from_str("#1b5e20"));
        ctx.set_font("bold 10px 'Segoe UI', Arial, sans-serif");
        let val_str = fmt_output_val(v0);
        let port_label = if !def.outputs.is_empty() { def.outputs[0].label } else { "→" };
        let _ = ctx.fill_text(&format!("{}: {}", port_label, val_str), body_cx, body_cy - 5.0);

        if let Some(v1) = node.outputs.get("1") {
            ctx.set_fill_style(&JsValue::from_str("#2e7d32"));
            ctx.set_font("9px 'Segoe UI', Arial, sans-serif");
            let label1 = if def.outputs.len() > 1 { def.outputs[1].label } else { "" };
            if !label1.is_empty() {
                let _ = ctx.fill_text(&format!("{}: {}", label1, fmt_output_val(v1)), body_cx, body_cy + 8.0);
            }
        }
    } else {
        // No outputs yet — show description
        ctx.set_fill_style(&JsValue::from_str("#546e7a"));
        ctx.set_font("10px 'Segoe UI', Arial, sans-serif");
        let _ = ctx.fill_text(def.description, body_cx, body_cy);
    }

    // Input ports (left edge)
    for (i, port) in def.inputs.iter().enumerate() {
        let (px, py) = abs_port_pos(node.kind, x, y, i, false);
        draw_port(ctx, px, py, false, port.label);
    }

    // Output ports (right edge)
    for (i, port) in def.outputs.iter().enumerate() {
        let (px, py) = abs_port_pos(node.kind, x, y, i, true);
        draw_port(ctx, px, py, true, port.label);
    }

    ctx.restore();
}

fn draw_port(ctx: &Ctx, cx: f64, cy: f64, is_output: bool, label: &str) {
    // Port circle
    ctx.begin_path();
    let _ = ctx.arc(cx, cy, PORT_R, 0.0, 2.0 * PI);
    ctx.set_fill_style(&JsValue::from_str("#ffffff"));
    ctx.set_stroke_style(&JsValue::from_str("#455a64"));
    ctx.set_line_width(1.5);
    ctx.fill();
    ctx.stroke();

    // Port label
    ctx.set_fill_style(&JsValue::from_str("#37474f"));
    ctx.set_font("9px 'Segoe UI', Arial, sans-serif");
    ctx.set_text_baseline("middle");
    if is_output {
        ctx.set_text_align("right");
        let _ = ctx.fill_text(label, cx - PORT_R - 3.0, cy);
    } else {
        ctx.set_text_align("left");
        let _ = ctx.fill_text(label, cx + PORT_R + 3.0, cy);
    }
}

fn draw_ghost_node(ctx: &Ctx, kind: NodeKind, wx: f64, wy: f64) {
    let def = kind.def();
    let x = wx - NODE_W / 2.0;
    let y = wy - NODE_H / 2.0;

    ctx.set_global_alpha(0.55);

    ctx.set_fill_style(&JsValue::from_str(def.category.body_fill()));
    ctx.set_stroke_style(&JsValue::from_str(def.category.border()));
    ctx.set_line_width(2.0);
    rounded_rect_path(ctx, x, y, NODE_W, NODE_H, CORNER_R);
    ctx.fill();
    ctx.stroke();

    ctx.set_fill_style(&JsValue::from_str(def.category.header_fill()));
    rounded_rect_top_path(ctx, x, y, NODE_W, HEADER_H, CORNER_R);
    ctx.fill();

    ctx.set_fill_style(&JsValue::from_str("#ffffff"));
    ctx.set_font("bold 12px 'Segoe UI', Arial, sans-serif");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(def.label, x + NODE_W / 2.0, y + HEADER_H / 2.0);

    ctx.set_global_alpha(1.0);
}

/// Path for a full rounded rectangle.
fn rounded_rect_path(ctx: &Ctx, x: f64, y: f64, w: f64, h: f64, r: f64) {
    ctx.begin_path();
    ctx.move_to(x + r, y);
    ctx.line_to(x + w - r, y);
    let _ = ctx.arc(x + w - r, y + r, r, -PI / 2.0, 0.0);
    ctx.line_to(x + w, y + h - r);
    let _ = ctx.arc(x + w - r, y + h - r, r, 0.0, PI / 2.0);
    ctx.line_to(x + r, y + h);
    let _ = ctx.arc(x + r, y + h - r, r, PI / 2.0, PI);
    ctx.line_to(x, y + r);
    let _ = ctx.arc(x + r, y + r, r, PI, 3.0 * PI / 2.0);
    ctx.close_path();
}

/// Path for top-rounded-only rectangle (header).
fn rounded_rect_top_path(ctx: &Ctx, x: f64, y: f64, w: f64, h: f64, r: f64) {
    ctx.begin_path();
    ctx.move_to(x + r, y);
    ctx.line_to(x + w - r, y);
    let _ = ctx.arc(x + w - r, y + r, r, -PI / 2.0, 0.0);
    ctx.line_to(x + w, y + h);
    ctx.line_to(x, y + h);
    ctx.line_to(x, y + r);
    let _ = ctx.arc(x + r, y + r, r, PI, 3.0 * PI / 2.0);
    ctx.close_path();
}

fn fmt_output_val(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.abs() >= 1000.0 { format!("{:.0}", f) }
            else if f.abs() >= 10.0 { format!("{:.1}", f) }
            else { format!("{:.3}", f) }
        }
        serde_json::Value::Bool(b) => if *b { "Yes".into() } else { "No".into() },
        serde_json::Value::String(s) => s.chars().take(14).collect(),
        _ => "…".into(),
    }
}

/// Hit test: which node is at (wx, wy)? Returns node_id.
pub fn hit_node(dag: &DagModel, wx: f64, wy: f64) -> Option<NodeId> {
    // Iterate in reverse so top-rendered nodes are hit first
    for node in dag.nodes.iter().rev() {
        if wx >= node.x && wx <= node.x + NODE_W
            && wy >= node.y && wy <= node.y + NODE_H
        {
            return Some(node.id);
        }
    }
    None
}

/// Hit test: which output port is at (wx, wy)?  Returns (node_id, port_idx).
pub fn hit_output_port(dag: &DagModel, wx: f64, wy: f64) -> Option<(NodeId, usize)> {
    use crate::nodes::PORT_HIT_R;
    for node in dag.nodes.iter().rev() {
        let n_out = node.kind.def().outputs.len();
        for i in 0..n_out {
            let (px, py) = abs_port_pos(node.kind, node.x, node.y, i, true);
            let dx = wx - px;
            let dy = wy - py;
            if dx * dx + dy * dy <= PORT_HIT_R * PORT_HIT_R {
                return Some((node.id, i));
            }
        }
    }
    None
}

/// Hit test: which input port is at (wx, wy)?  Returns (node_id, port_idx).
pub fn hit_input_port(dag: &DagModel, wx: f64, wy: f64) -> Option<(NodeId, usize)> {
    use crate::nodes::PORT_HIT_R;
    for node in dag.nodes.iter().rev() {
        let n_in = node.kind.def().inputs.len();
        for i in 0..n_in {
            let (px, py) = abs_port_pos(node.kind, node.x, node.y, i, false);
            let dx = wx - px;
            let dy = wy - py;
            if dx * dx + dy * dy <= PORT_HIT_R * PORT_HIT_R {
                return Some((node.id, i));
            }
        }
    }
    None
}

/// Hit test: which edge is near (wx, wy)?
pub fn hit_edge(dag: &DagModel, wx: f64, wy: f64) -> Option<EdgeId> {
    const THRESHOLD: f64 = 8.0;
    for edge in dag.edges.iter().rev() {
        let fn_ = dag.get_node(edge.from_node)?;
        let tn = dag.get_node(edge.to_node)?;
        let (x1, y1) = abs_port_pos(fn_.kind, fn_.x, fn_.y, edge.from_port, true);
        let (x2, y2) = abs_port_pos(tn.kind, tn.x, tn.y, edge.to_port, false);
        if point_near_bezier(wx, wy, x1, y1, x2, y2, THRESHOLD) {
            return Some(edge.id);
        }
    }
    None
}

/// Sample a cubic bezier at ~20 points and check distance to (px, py).
fn point_near_bezier(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64, threshold: f64) -> bool {
    let dx = (x2 - x1).abs().max(60.0) * 0.55;
    let cp1x = x1 + dx;
    let cp1y = y1;
    let cp2x = x2 - dx;
    let cp2y = y2;
    for i in 0..=20 {
        let t = i as f64 / 20.0;
        let mt = 1.0 - t;
        let bx = mt.powi(3) * x1 + 3.0 * mt.powi(2) * t * cp1x + 3.0 * mt * t.powi(2) * cp2x + t.powi(3) * x2;
        let by = mt.powi(3) * y1 + 3.0 * mt.powi(2) * t * cp1y + 3.0 * mt * t.powi(2) * cp2y + t.powi(3) * y2;
        if (bx - px).powi(2) + (by - py).powi(2) <= threshold.powi(2) {
            return true;
        }
    }
    false
}
