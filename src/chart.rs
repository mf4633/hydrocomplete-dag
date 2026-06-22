/// Pure-canvas chart renderer — no Chart.js dependency.
/// Draws line, bar, and gauge charts directly onto a 2D canvas
/// from data stored in node outputs after a DAG run.
use std::f64::consts::PI;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d as Ctx;

struct Pad { l: f64, r: f64, t: f64, b: f64 }

// ── Palette ───────────────────────────────────────────────────────────────────
const COL_GRID: &str   = "#30363d";
const COL_AXIS: &str   = "#8b949e";
const COL_TEXT: &str   = "#c9d1d9";
const COL_BG: &str     = "#0d1117";
const COL_LINE: &str   = "#1f6feb";
const COL_FILL: &str   = "rgba(31,111,235,0.18)";
const COL_PEAK: &str   = "#f78166";
const COL_TSS: &str    = "#e67e22";
const COL_TN: &str     = "#27ae60";
const COL_TP: &str     = "#2980b9";
const COL_BAR: [&str; 6] = ["#1f6feb","#238636","#da3633","#e3b341","#8957e5","#3fb950"];

// ── Public entry points ───────────────────────────────────────────────────────

/// Dispatch chart rendering based on node kind + available output keys.
pub fn render_node_chart(ctx: &Ctx, w: f64, h: f64, kind: &str, outputs: &serde_json::Value) {
    clear(ctx, w, h);
    match kind {
        "detention_pond" | "pond_chain" => draw_hydrograph(ctx, w, h, outputs),
        "continuous_sim"                => draw_monthly_bars(ctx, w, h, outputs),
        "treatment_train" | "treatment_bmp" => draw_pollutant_bars(ctx, w, h, outputs),
        "rusle_erosion"                 => draw_erosion_gauge(ctx, w, h, outputs),
        "manning_pipe"                  => draw_capacity_gauge(ctx, w, h, outputs),
        "scs_runoff" | "loss_method"    => draw_runoff_simple(ctx, w, h, outputs),
        "water_quality_volume"          => draw_wqv_simple(ctx, w, h, outputs),
        "rational_method"               => draw_rational_simple(ctx, w, h, outputs),
        _                               => draw_placeholder(ctx, w, h, "Select a computed node"),
    }
}

// ── Hydrograph line chart ─────────────────────────────────────────────────────

fn draw_hydrograph(ctx: &Ctx, w: f64, h: f64, outs: &serde_json::Value) {
    let inflow  = float_array(outs, "hydro_inflow_cfs");
    let outflow = float_array(outs, "hydro_outflow_cfs");
    let peak_out = outs["0"].as_f64().unwrap_or(0.0);
    let stor     = outs["peak_storage_cf"].as_f64().unwrap_or(0.0);
    let atten    = outs["attenuation_pct"].as_f64().unwrap_or(0.0);

    if inflow.is_empty() && outflow.is_empty() {
        // No array data yet — show scalar summary
        draw_kv_panel(ctx, w, h, "Detention Routing Result", &[
            ("Peak Q out", &fmt_val(peak_out), "cfs"),
            ("Attenuation", &format!("{:.1}", atten), "%"),
            ("Peak storage", &fmt_val(stor), "cf"),
        ]);
        return;
    }

    let pad = Pad { l: 52.0, r: 18.0, t: 28.0, b: 38.0 };
    let px = pad.l; let py = pad.t;
    let pw = w - pad.l - pad.r;
    let ph = h - pad.t - pad.b;

    let max_q = inflow.iter().chain(outflow.iter()).cloned().fold(0.0_f64, f64::max).max(1.0);
    let n = inflow.len().max(outflow.len()).max(1);

    let mx = |i: usize| px + (i as f64) / (n as f64 - 1.0).max(1.0) * pw;
    let my = |q: f64|   py + ph - (q / max_q) * ph;

    // Grid
    draw_grid(ctx, px, py, pw, ph, 4, 4);

    // Axes
    axis_left(ctx, px, py, ph, 0.0, max_q, 4, "cfs");
    axis_bottom(ctx, px, py + ph, pw, n, "hr");

    // Inflow area fill
    if inflow.len() > 1 {
        ctx.begin_path();
        ctx.move_to(mx(0), my(0.0));
        for (i, &v) in inflow.iter().enumerate() { ctx.line_to(mx(i), my(v)); }
        ctx.line_to(mx(inflow.len() - 1), my(0.0));
        ctx.close_path();
        ctx.set_fill_style(&JsValue::from_str("rgba(100,120,180,0.12)"));
        ctx.fill();

        ctx.begin_path();
        for (i, &v) in inflow.iter().enumerate() {
            if i == 0 { ctx.move_to(mx(0), my(v)); } else { ctx.line_to(mx(i), my(v)); }
        }
        ctx.set_stroke_style(&JsValue::from_str("#6475b4"));
        ctx.set_line_width(1.5);
        let _ = ctx.set_line_dash(&js_sys::Array::new());
        ctx.stroke();
    }

    // Outflow line
    if outflow.len() > 1 {
        ctx.begin_path();
        ctx.set_fill_style(&JsValue::from_str(COL_FILL));
        ctx.move_to(mx(0), my(0.0));
        for (i, &v) in outflow.iter().enumerate() { ctx.line_to(mx(i), my(v)); }
        ctx.line_to(mx(outflow.len() - 1), my(0.0));
        ctx.close_path();
        ctx.fill();

        ctx.begin_path();
        for (i, &v) in outflow.iter().enumerate() {
            if i == 0 { ctx.move_to(mx(0), my(v)); } else { ctx.line_to(mx(i), my(v)); }
        }
        ctx.set_stroke_style(&JsValue::from_str(COL_LINE));
        ctx.set_line_width(2.0);
        ctx.stroke();
    }

    // Peak marker
    if outflow.len() > 1 {
        let (pi, pq) = outflow.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap_or((0, &0.0));
        let ppx = mx(pi); let ppy = my(*pq);
        ctx.begin_path();
        let _ = ctx.arc(ppx, ppy, 4.0, 0.0, 2.0 * PI);
        ctx.set_fill_style(&JsValue::from_str(COL_PEAK));
        ctx.fill();
        label(ctx, &fmt_val(*pq), ppx + 6.0, ppy - 6.0, COL_PEAK, 9.0, "left");
    }

    // Legend
    legend(ctx, w - 110.0, py + 4.0, &[("#6475b4","Inflow"),(COL_LINE,"Outflow")]);
    title(ctx, "Detention Hydrograph", w / 2.0, py - 10.0);
}

// ── Monthly bar chart ─────────────────────────────────────────────────────────

fn draw_monthly_bars(ctx: &Ctx, w: f64, h: f64, outs: &serde_json::Value) {
    const MONTHS: [&str; 12] = ["J","F","M","A","M","J","J","A","S","O","N","D"];

    let rain   = float_array(outs, "monthly_rain_in");
    let runoff = float_array(outs, "monthly_runoff_ac_in");
    let tss    = float_array(outs, "monthly_tss_lbs");

    if rain.is_empty() {
        // Scalar fallback
        let q_ann  = outs["0"].as_f64().unwrap_or(0.0);
        let tss_yr = outs["tss_lbs_yr"].as_f64().unwrap_or(0.0);
        let tn_yr  = outs["tn_lbs_yr"].as_f64().unwrap_or(0.0);
        draw_kv_panel(ctx, w, h, "Continuous Simulation", &[
            ("Annual runoff", &fmt_val(q_ann), "ac-in/yr"),
            ("Annual TSS load", &fmt_val(tss_yr), "lbs/yr"),
            ("Annual TN load", &fmt_val(tn_yr), "lbs/yr"),
        ]);
        return;
    }

    let n = 12.min(rain.len());
    let pad = Pad { l: 44.0, r: 10.0, t: 28.0, b: 30.0 };
    let pw = w - pad.l - pad.r;
    let ph = h - pad.t - pad.b;
    let bar_w = (pw / n as f64) * 0.4;
    let max_r = rain.iter().cloned().fold(0.0_f64, f64::max).max(0.01);

    draw_grid(ctx, pad.l, pad.t, pw, ph, 4, 0);
    axis_left(ctx, pad.l, pad.t, ph, 0.0, max_r, 4, "in");

    for i in 0..n {
        let cx = pad.l + (i as f64 + 0.5) / n as f64 * pw;
        let bar_h = (rain[i] / max_r) * ph;
        let bx = cx - bar_w;
        ctx.set_fill_style(&JsValue::from_str("#1f6feb99"));
        ctx.fill_rect(bx, pad.t + ph - bar_h, bar_w, bar_h);

        if !runoff.is_empty() && i < runoff.len() {
            let rh = (runoff[i] / max_r) * ph;
            ctx.set_fill_style(&JsValue::from_str("#238636cc"));
            ctx.fill_rect(cx, pad.t + ph - rh, bar_w, rh);
        }

        label(ctx, MONTHS[i], cx - bar_w / 2.0, pad.t + ph + 12.0, COL_AXIS, 9.0, "center");
    }

    legend(ctx, w - 120.0, pad.t + 2.0, &[("#1f6feb","Rainfall"),(  "#238636","Runoff")]);
    title(ctx, "Monthly Water Balance", w / 2.0, pad.t - 10.0);
}

// ── Pollutant removal grouped bars ────────────────────────────────────────────

fn draw_pollutant_bars(ctx: &Ctx, w: f64, h: f64, outs: &serde_json::Value) {
    let tss_in  = outs["tss_in_lbs"].as_f64().unwrap_or(0.0);
    let tn_in   = outs["tn_in_lbs"].as_f64().unwrap_or(0.0);
    let tp_in   = outs["tp_in_lbs"].as_f64().unwrap_or(0.0);
    let tss_out = outs["tss_out_lbs"].as_f64().unwrap_or(0.0);
    let tn_out  = outs["tn_out_lbs"].as_f64().unwrap_or(0.0);
    let tp_out  = outs["tp_out_lbs"].as_f64().unwrap_or(0.0);
    let eta     = outs["1"].as_f64().unwrap_or(0.0);

    if tss_in == 0.0 && tn_in == 0.0 {
        draw_kv_panel(ctx, w, h, "Treatment Train", &[
            ("TSS η", &format!("{:.1}", eta), "%"),
        ]);
        return;
    }

    let labels   = ["TSS",    "TN",   "TP"];
    let influent = [tss_in,   tn_in,  tp_in];
    let effluent = [tss_out,  tn_out, tp_out];
    let colors   = [COL_TSS, COL_TN, COL_TP];

    let pad = Pad { l: 44.0, r: 14.0, t: 28.0, b: 36.0 };
    let pw  = w - pad.l - pad.r;
    let ph  = h - pad.t - pad.b;
    let max_v = influent.iter().cloned().fold(0.0_f64, f64::max).max(0.01);
    let group_w = pw / 3.0;
    let bar_w   = group_w * 0.3;

    draw_grid(ctx, pad.l, pad.t, pw, ph, 4, 0);
    axis_left(ctx, pad.l, pad.t, ph, 0.0, max_v, 4, "lbs");

    for i in 0..3 {
        let gx = pad.l + (i as f64 + 0.5) * group_w;
        // Influent
        let ih = (influent[i] / max_v) * ph;
        ctx.set_fill_style(&JsValue::from_str(&format!("{}aa", colors[i])));
        ctx.fill_rect(gx - bar_w - 1.0, pad.t + ph - ih, bar_w, ih);
        // Effluent
        let eh = (effluent[i] / max_v) * ph;
        ctx.set_fill_style(&JsValue::from_str(colors[i]));
        ctx.fill_rect(gx + 1.0, pad.t + ph - eh, bar_w, eh);
        // Removal %
        let rem_pct = if influent[i] > 0.0 { (1.0 - effluent[i] / influent[i]) * 100.0 } else { 0.0 };
        label(ctx, &format!("{:.0}%", rem_pct), gx, pad.t + ph + 14.0, COL_AXIS, 9.0, "center");
        label(ctx, labels[i], gx, pad.t + ph + 24.0, COL_TEXT, 9.0, "center");
    }

    legend(ctx, w - 130.0, pad.t + 2.0, &[("rgba(200,150,60,0.67)","Influent"),("rgba(200,150,60,1.0)","Effluent")]);
    title(ctx, "Pollutant Removal", w / 2.0, pad.t - 10.0);
}

// ── Erosion gauge ─────────────────────────────────────────────────────────────

fn draw_erosion_gauge(ctx: &Ctx, w: f64, h: f64, outs: &serde_json::Value) {
    let a = outs["0"].as_f64().unwrap_or(0.0);
    let total = outs["total_tons_yr"].as_f64().unwrap_or(0.0);

    let cx = w / 2.0;
    let cy = h * 0.55;
    let r  = (w.min(h) * 0.35).min(80.0);

    // Risk tiers (t/ac/yr): <2 Low, 2-5 Moderate, 5-10 High, >10 Very High
    let tiers: [(&str, &str, f64); 4] = [
        ("#238636", "Low",       2.0),
        ("#e3b341", "Moderate",  5.0),
        ("#da3633", "High",     10.0),
        ("#8957e5", "Very High", f64::INFINITY),
    ];

    // Draw arc segments
    let start = PI;
    let total_arc = PI;
    let max_disp = 15.0_f64;
    let arc_per_unit = total_arc / max_disp;

    let mut arc_start = start;
    let mut prev_cap = 0.0_f64;
    for (col, _, cap) in &tiers {
        let seg = ((cap.min(max_disp) - prev_cap) * arc_per_unit).max(0.0);
        let seg = if cap.is_infinite() { total_arc - (arc_start - start) } else { seg };
        ctx.begin_path();
        let _ = ctx.arc(cx, cy, r, arc_start, arc_start + seg);
        let _ = ctx.arc_with_anticlockwise(cx, cy, r * 0.6, arc_start + seg, arc_start, true);
        ctx.close_path();
        ctx.set_fill_style(&JsValue::from_str(col));
        ctx.fill();
        arc_start += seg;
        prev_cap = cap.min(max_disp);
    }

    // Needle
    let needle_frac = (a / max_disp).min(1.0);
    let angle = start + needle_frac * total_arc;
    let nx = cx + r * 0.72 * angle.cos();
    let ny = cy + r * 0.72 * angle.sin();
    ctx.begin_path();
    ctx.move_to(cx, cy);
    ctx.line_to(nx, ny);
    ctx.set_stroke_style(&JsValue::from_str("#fff"));
    ctx.set_line_width(2.5);
    ctx.stroke();
    ctx.begin_path();
    let _ = ctx.arc(cx, cy, 5.0, 0.0, 2.0 * PI);
    ctx.set_fill_style(&JsValue::from_str("#fff"));
    ctx.fill();

    // Risk label
    let risk = if a < 2.0 { "Low" } else if a < 5.0 { "Moderate" } else if a < 10.0 { "High" } else { "Very High" };
    let risk_col = if a < 2.0 { "#238636" } else if a < 5.0 { "#e3b341" } else if a < 10.0 { "#da3633" } else { "#8957e5" };
    label(ctx, risk, cx, cy + r * 0.25, risk_col, 13.0, "center");
    label(ctx, &format!("{:.2} t/ac/yr", a), cx, cy + r * 0.45, COL_TEXT, 10.0, "center");
    if total > 0.0 {
        label(ctx, &format!("Total: {:.1} t/yr", total), cx, cy + r * 0.62, COL_AXIS, 9.0, "center");
    }
    title(ctx, "RUSLE Soil Loss Risk", cx, pad_t(h));
}

// ── Pipe capacity gauge ───────────────────────────────────────────────────────

fn draw_capacity_gauge(ctx: &Ctx, w: f64, h: f64, outs: &serde_json::Value) {
    let q_full = outs["0"].as_f64().unwrap_or(0.0);
    let depth  = outs["1"].as_f64().unwrap_or(0.0);
    let ratio  = outs["q_ratio"].as_f64().unwrap_or(0.0);
    let surch  = outs["surcharged"].as_f64().unwrap_or(0.0) > 0.5;

    let cx = w / 2.0; let cy = h * 0.5; let r = (w.min(h) * 0.32).min(75.0);

    // Background circle
    ctx.begin_path();
    let _ = ctx.arc(cx, cy, r, 0.0, 2.0 * PI);
    ctx.set_stroke_style(&JsValue::from_str(COL_GRID));
    ctx.set_line_width(12.0);
    ctx.stroke();

    // Fill arc
    let fill_col = if surch { "#da3633" } else if ratio > 0.85 { "#e3b341" } else { "#238636" };
    let end_angle = -PI / 2.0 + ratio.min(1.0) * 2.0 * PI;
    ctx.begin_path();
    let _ = ctx.arc(cx, cy, r, -PI / 2.0, end_angle);
    ctx.set_stroke_style(&JsValue::from_str(fill_col));
    ctx.set_line_width(12.0);
    ctx.stroke();

    let status = if surch { "SURCHARGED" } else if ratio > 0.85 { "NEAR FULL" } else { "OK" };
    label(ctx, &format!("{:.0}%", ratio * 100.0), cx, cy - 10.0, fill_col, 22.0, "center");
    label(ctx, "Q/Qfull", cx, cy + 14.0, COL_AXIS, 9.0, "center");
    label(ctx, status, cx, cy + 28.0, fill_col, 10.0, "center");

    // Stats below
    label(ctx, &format!("Qfull = {} cfs", fmt_val(q_full)), cx, cy + r + 20.0, COL_TEXT, 9.0, "center");
    label(ctx, &format!("d = {} ft", fmt_val(depth)), cx, cy + r + 32.0, COL_TEXT, 9.0, "center");
    title(ctx, "Pipe Capacity", cx, pad_t(h));
}

// ── Simple scalar displays ────────────────────────────────────────────────────

fn draw_runoff_simple(ctx: &Ctx, w: f64, h: f64, outs: &serde_json::Value) {
    let q = outs["0"].as_f64().unwrap_or(0.0);
    let vol = outs["volume_cf"].as_f64().unwrap_or(0.0);
    let loss = outs["loss_in"].as_f64().unwrap_or(0.0);
    draw_kv_panel(ctx, w, h, "Runoff", &[
        ("Excess depth", &fmt_val(q),   "in"),
        ("Volume",       &fmt_val(vol), "cf"),
        ("Total loss",   &fmt_val(loss),"in"),
    ]);
}

fn draw_wqv_simple(ctx: &Ctx, w: f64, h: f64, outs: &serde_json::Value) {
    let wqv = outs["0"].as_f64().unwrap_or(0.0);
    let rv  = outs["rv"].as_f64().unwrap_or(0.0);
    draw_kv_panel(ctx, w, h, "Water Quality Volume", &[
        ("WQV",    &fmt_val(wqv), "cf"),
        ("Rv",     &format!("{:.3}", rv), ""),
    ]);
}

fn draw_rational_simple(ctx: &Ctx, w: f64, h: f64, outs: &serde_json::Value) {
    let q = outs["0"].as_f64().unwrap_or(0.0);
    draw_kv_panel(ctx, w, h, "Rational Method", &[
        ("Peak flow Q", &fmt_val(q), "cfs"),
    ]);
}

fn draw_placeholder(ctx: &Ctx, w: f64, h: f64, msg: &str) {
    label(ctx, msg, w / 2.0, h / 2.0, COL_AXIS, 11.0, "center");
}

// ── KV panel ─────────────────────────────────────────────────────────────────

fn draw_kv_panel(ctx: &Ctx, w: f64, h: f64, heading: &str, rows: &[(&str, &str, &str)]) {
    title(ctx, heading, w / 2.0, 20.0);
    let row_h = 28.0;
    let start_y = h / 2.0 - (rows.len() as f64 * row_h) / 2.0;
    for (i, (k, v, unit)) in rows.iter().enumerate() {
        let y = start_y + i as f64 * row_h;
        label(ctx, k, 12.0, y, COL_AXIS, 11.0, "left");
        label(ctx, &format!("{} {}", v, unit), w - 12.0, y, COL_TEXT, 12.0, "right");
    }
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

fn clear(ctx: &Ctx, w: f64, h: f64) {
    let _ = ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    ctx.set_fill_style(&JsValue::from_str(COL_BG));
    ctx.fill_rect(0.0, 0.0, w, h);
}

fn draw_grid(ctx: &Ctx, x: f64, y: f64, w: f64, h: f64, ny: usize, _nx: usize) {
    ctx.set_stroke_style(&JsValue::from_str(COL_GRID));
    ctx.set_line_width(0.5);
    for i in 0..=ny {
        let gy = y + h * (1.0 - i as f64 / ny as f64);
        ctx.begin_path();
        ctx.move_to(x, gy);
        ctx.line_to(x + w, gy);
        ctx.stroke();
    }
}

fn axis_left(ctx: &Ctx, x: f64, y: f64, h: f64, min: f64, max: f64, ticks: usize, unit: &str) {
    for i in 0..=ticks {
        let v = min + (max - min) * i as f64 / ticks as f64;
        let gy = y + h - h * i as f64 / ticks as f64;
        label(ctx, &fmt_axis(v), x - 4.0, gy, COL_AXIS, 8.0, "right");
    }
    label(ctx, unit, x - 4.0, y - 8.0, COL_AXIS, 8.0, "right");
}

fn axis_bottom(ctx: &Ctx, x: f64, y: f64, w: f64, n: usize, unit: &str) {
    let step = (n / 4).max(1);
    for i in (0..n).step_by(step) {
        let gx = x + w * i as f64 / (n as f64 - 1.0).max(1.0);
        label(ctx, &i.to_string(), gx, y + 12.0, COL_AXIS, 8.0, "center");
    }
    label(ctx, unit, x + w + 4.0, y + 12.0, COL_AXIS, 8.0, "left");
}

fn legend(ctx: &Ctx, x: f64, y: f64, items: &[(&str, &str)]) {
    for (i, (col, name)) in items.iter().enumerate() {
        let lx = x; let ly = y + i as f64 * 14.0;
        ctx.set_fill_style(&JsValue::from_str(col));
        ctx.fill_rect(lx, ly, 10.0, 9.0);
        label(ctx, name, lx + 13.0, ly + 4.0, COL_AXIS, 9.0, "left");
    }
}

fn title(ctx: &Ctx, text: &str, x: f64, y: f64) {
    label(ctx, text, x, y, COL_TEXT, 11.0, "center");
}

fn label(ctx: &Ctx, text: &str, x: f64, y: f64, color: &str, size: f64, align: &str) {
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.set_font(&format!("{}px 'Segoe UI', Arial, sans-serif", size));
    ctx.set_text_align(align);
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(text, x, y);
}

fn pad_t(h: f64) -> f64 { h * 0.08 }

// ── Data helpers ──────────────────────────────────────────────────────────────

fn float_array(outs: &serde_json::Value, key: &str) -> Vec<f64> {
    outs[key].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default()
}

fn fmt_val(v: f64) -> String {
    if v >= 10000.0 { format!("{:.0}", v) }
    else if v >= 100.0 { format!("{:.1}", v) }
    else { format!("{:.3}", v) }
}

fn fmt_axis(v: f64) -> String {
    if v >= 1000.0 { format!("{:.0}", v) }
    else if v >= 10.0 { format!("{:.1}", v) }
    else { format!("{:.2}", v) }
}
