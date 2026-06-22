/// Pre-built DAG templates for common stormwater design workflows.
use crate::dag::DagModel;
use crate::nodes::NodeKind;

// Horizontal spacing between nodes
const DX: f64 = 220.0;
// Default config helpers
use std::collections::HashMap;
use serde_json::Value;

fn num(v: f64) -> Value { Value::Number(serde_json::Number::from_f64(v).unwrap()) }
fn str_val(s: &str) -> Value { Value::String(s.to_string()) }

fn cfg(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

// ── Template definitions ──────────────────────────────────────────────────────

/// Detention pond sizing: Rainfall → SCS → Unit Hydro → Detention → Outfall
pub fn detention_design() -> DagModel {
    let mut dag = DagModel::new();
    let y = 120.0;

    let rain = dag.add_node(NodeKind::RainfallEvent, 30.0, y);
    if let Some(n) = dag.get_node_mut(rain) {
        n.config = cfg(&[("depth_in", num(4.2)), ("duration_hr", num(24.0))]);
        n.label = Some("10-yr Storm".to_string());
    }

    let catch = dag.add_node(NodeKind::Catchment, 30.0, y + 160.0);
    if let Some(n) = dag.get_node_mut(catch) {
        n.config = cfg(&[("area_acres", num(15.0)), ("curve_number", num(78.0)),
                         ("land_use", str_val("residential-medium"))]);
    }

    let scs = dag.add_node(NodeKind::ScsRunoff, 30.0 + DX, y);
    if let Some(n) = dag.get_node_mut(scs) {
        n.config = cfg(&[("curve_number", num(78.0)), ("area_acres", num(15.0))]);
    }

    let uh = dag.add_node(NodeKind::UnitHydrograph, 30.0 + DX * 2.0, y);
    if let Some(n) = dag.get_node_mut(uh) {
        n.config = cfg(&[("area_acres", num(15.0)), ("tc_min", num(25.0))]);
    }

    let det = dag.add_node(NodeKind::DetentionPond, 30.0 + DX * 3.0, y);
    if let Some(n) = dag.get_node_mut(det) {
        n.config = cfg(&[
            ("area_acres", num(15.0)), ("curve_number", num(78.0)),
            ("rainfall_in", num(4.2)), ("tc_min", num(25.0)),
            ("bottom_area_sf", num(8000.0)), ("side_slope", num(3.0)),
            ("max_depth_ft", num(6.0)), ("orifice_dia_in", num(6.0)),
        ]);
    }

    let out = dag.add_node(NodeKind::Outfall, 30.0 + DX * 4.0, y);

    dag.add_edge(rain,  0, scs, 1);
    dag.add_edge(catch, 0, scs, 0);
    dag.add_edge(scs,   0, uh,  0);
    dag.add_edge(uh,    0, det, 0);
    dag.add_edge(det,   0, out, 0);

    dag
}

/// Water quality design: Catchment → WQV → BMP Sizing → Treatment → Outfall
pub fn wq_design() -> DagModel {
    let mut dag = DagModel::new();
    let y = 120.0;

    let catch = dag.add_node(NodeKind::Catchment, 30.0, y);
    if let Some(n) = dag.get_node_mut(catch) {
        n.config = cfg(&[("area_acres", num(8.0)), ("curve_number", num(80.0)),
                         ("land_use", str_val("commercial"))]);
    }

    let storm = dag.add_node(NodeKind::RainfallEvent, 30.0, y + 160.0);
    if let Some(n) = dag.get_node_mut(storm) {
        n.config = cfg(&[("depth_in", num(1.0)), ("duration_hr", num(24.0))]);
        n.label = Some("WQ Storm 1\"".to_string());
    }

    let wqv = dag.add_node(NodeKind::WaterQualityVolume, 30.0 + DX, y);
    if let Some(n) = dag.get_node_mut(wqv) {
        n.config = cfg(&[("area_acres", num(8.0)), ("impervious_pct", num(65.0)),
                         ("design_storm_in", num(1.0))]);
    }

    let sizing = dag.add_node(NodeKind::BmpSizing, 30.0 + DX * 2.0, y);
    if let Some(n) = dag.get_node_mut(sizing) {
        n.config = cfg(&[("bmp_type", str_val("bioretention")),
                         ("area_acres", num(8.0)), ("impervious_pct", num(65.0)),
                         ("design_storm_in", num(1.0))]);
    }

    let treat = dag.add_node(NodeKind::TreatmentTrain, 30.0 + DX * 3.0, y);
    if let Some(n) = dag.get_node_mut(treat) {
        n.config = cfg(&[("bmp_chain", str_val("bioretention,wet-pond")),
                         ("area_acres", num(8.0)),
                         ("land_use", str_val("commercial")),
                         ("runoff_in", num(0.5))]);
    }

    let out = dag.add_node(NodeKind::Outfall, 30.0 + DX * 4.0, y);

    dag.add_edge(catch, 0, wqv, 0);
    dag.add_edge(storm, 0, wqv, 1);
    dag.add_edge(wqv,   0, sizing, 0);
    dag.add_edge(sizing,0, treat, 0);
    dag.add_edge(treat, 0, out, 0);

    dag
}

/// Erosion & sediment control: Catchment → RUSLE → Sediment Basin → Outfall
pub fn erosion_control() -> DagModel {
    let mut dag = DagModel::new();
    let y = 120.0;

    let catch = dag.add_node(NodeKind::Catchment, 30.0, y);
    if let Some(n) = dag.get_node_mut(catch) {
        n.config = cfg(&[("area_acres", num(5.0)), ("curve_number", num(85.0)),
                         ("land_use", str_val("construction"))]);
    }

    let rat = dag.add_node(NodeKind::RationalMethod, 30.0 + DX, y - 80.0);
    if let Some(n) = dag.get_node_mut(rat) {
        n.config = cfg(&[("area_acres", num(5.0)), ("runoff_c", num(0.85)),
                         ("intensity_in_hr", num(3.5))]);
    }

    let rusle = dag.add_node(NodeKind::RusleErosion, 30.0 + DX, y + 80.0);
    if let Some(n) = dag.get_node_mut(rusle) {
        n.config = cfg(&[("region", str_val("charlotte-nc")),
                         ("area_acres", num(5.0)), ("slope_length_ft", num(150.0)),
                         ("slope_pct", num(8.0))]);
        n.label = Some("Site Erosion".to_string());
    }

    let basin = dag.add_node(NodeKind::SedimentBasin, 30.0 + DX * 2.0, y);
    if let Some(n) = dag.get_node_mut(basin) {
        n.config = cfg(&[("area_acres", num(5.0)), ("design_q_cfs", num(12.0)),
                         ("sed_yield_tons_ac_yr", num(25.0))]);
    }

    let out = dag.add_node(NodeKind::Outfall, 30.0 + DX * 3.0, y);

    dag.add_edge(catch, 0, rat,   0);
    dag.add_edge(catch, 0, rusle, 0);
    dag.add_edge(rat,   0, basin, 0);
    dag.add_edge(rusle, 0, basin, 1);
    dag.add_edge(basin, 0, out,   0);

    dag
}

/// Full stormwater: Hydrology → Hydraulics → WQ all in one chain
pub fn full_stormwater() -> DagModel {
    let mut dag = DagModel::new();

    // Row 1: Hydrology
    let catch = dag.add_node(NodeKind::Catchment, 30.0, 60.0);
    if let Some(n) = dag.get_node_mut(catch) {
        n.config = cfg(&[("area_acres", num(20.0)), ("curve_number", num(76.0)),
                         ("land_use", str_val("residential-medium"))]);
    }
    let rain = dag.add_node(NodeKind::RainfallEvent, 30.0, 200.0);
    if let Some(n) = dag.get_node_mut(rain) {
        n.config = cfg(&[("depth_in", num(3.5))]);
        n.label = Some("2-yr, 24-hr".to_string());
    }
    let scs = dag.add_node(NodeKind::ScsRunoff, 30.0 + DX, 60.0);
    if let Some(n) = dag.get_node_mut(scs) {
        n.config = cfg(&[("curve_number", num(76.0)), ("area_acres", num(20.0))]);
    }
    let tc = dag.add_node(NodeKind::TimeOfConcentration, 30.0 + DX, 200.0);
    if let Some(n) = dag.get_node_mut(tc) {
        n.config = cfg(&[("tc_min", num(22.0))]);
    }
    let uh = dag.add_node(NodeKind::UnitHydrograph, 30.0 + DX * 2.0, 60.0);
    if let Some(n) = dag.get_node_mut(uh) {
        n.config = cfg(&[("area_acres", num(20.0)), ("tc_min", num(22.0))]);
    }

    // Row 2: Hydraulics
    let pipe = dag.add_node(NodeKind::ManningPipe, 30.0 + DX * 2.0, 240.0);
    if let Some(n) = dag.get_node_mut(pipe) {
        n.config = cfg(&[("diameter_ft", num(2.0)), ("slope", num(0.005)),
                         ("manning_n", num(0.013))]);
        n.label = Some("Outlet Pipe".to_string());
    }
    let det = dag.add_node(NodeKind::DetentionPond, 30.0 + DX * 3.0, 60.0);
    if let Some(n) = dag.get_node_mut(det) {
        n.config = cfg(&[("area_acres", num(20.0)), ("curve_number", num(76.0)),
                         ("rainfall_in", num(3.5)), ("tc_min", num(22.0)),
                         ("bottom_area_sf", num(12000.0)), ("side_slope", num(3.0)),
                         ("max_depth_ft", num(8.0)), ("orifice_dia_in", num(8.0))]);
    }

    // Row 3: WQ
    let wqv = dag.add_node(NodeKind::WaterQualityVolume, 30.0 + DX * 3.0, 240.0);
    if let Some(n) = dag.get_node_mut(wqv) {
        n.config = cfg(&[("area_acres", num(20.0)), ("impervious_pct", num(45.0)),
                         ("design_storm_in", num(1.0))]);
    }
    let treat = dag.add_node(NodeKind::TreatmentBmp, 30.0 + DX * 4.0, 240.0);
    if let Some(n) = dag.get_node_mut(treat) {
        n.config = cfg(&[("bmp_type", str_val("bioretention")),
                         ("area_acres", num(20.0)), ("land_use", str_val("residential-medium")),
                         ("runoff_in", num(0.5))]);
    }
    let out = dag.add_node(NodeKind::Outfall, 30.0 + DX * 5.0, 150.0);

    // Edges
    dag.add_edge(rain,  0, scs, 1);
    dag.add_edge(catch, 0, scs, 0);
    dag.add_edge(catch, 0, tc,  0);
    dag.add_edge(scs,   0, uh,  0);
    dag.add_edge(tc,    0, uh,  1);
    dag.add_edge(uh,    0, det, 0);
    dag.add_edge(scs,   0, pipe,0);
    dag.add_edge(det,   0, out, 0);
    dag.add_edge(catch, 0, wqv, 0);
    dag.add_edge(rain,  0, wqv, 1);
    dag.add_edge(wqv,   0, treat,0);
    dag.add_edge(treat, 0, out, 0); // may fail (port occupied)

    dag
}

/// Continuous simulation: long-term pollutant load assessment
pub fn continuous_sim() -> DagModel {
    let mut dag = DagModel::new();
    let y = 120.0;

    let site = dag.add_node(NodeKind::Catchment, 30.0, y);
    if let Some(n) = dag.get_node_mut(site) {
        n.config = cfg(&[("area_acres", num(10.0)), ("curve_number", num(74.0)),
                         ("land_use", str_val("residential-medium"))]);
        n.label = Some("Site".to_string());
    }

    let sim = dag.add_node(NodeKind::ContinuousSim, 30.0 + DX, y);
    if let Some(n) = dag.get_node_mut(sim) {
        n.config = cfg(&[("location", str_val("charlotte-nc")),
                         ("area_acres", num(10.0)), ("curve_number", num(74.0)),
                         ("land_use", str_val("residential-medium")), ("years", num(5.0))]);
        n.label = Some("5-yr Sim".to_string());
    }

    let treat = dag.add_node(NodeKind::TreatmentTrain, 30.0 + DX * 2.0, y);
    if let Some(n) = dag.get_node_mut(treat) {
        n.config = cfg(&[("bmp_chain", str_val("bioretention,constructed-wetland")),
                         ("area_acres", num(10.0)), ("land_use", str_val("residential-medium")),
                         ("runoff_in", num(0.5))]);
    }

    let out = dag.add_node(NodeKind::Outfall, 30.0 + DX * 3.0, y);

    dag.add_edge(site,  0, sim,   0);
    dag.add_edge(sim,   0, treat, 0);
    dag.add_edge(treat, 0, out,   0);

    dag
}

// ── Public lookup ─────────────────────────────────────────────────────────────

pub fn get(name: &str) -> Option<DagModel> {
    match name {
        "detention"   => Some(detention_design()),
        "wq"          => Some(wq_design()),
        "erosion"     => Some(erosion_control()),
        "full"        => Some(full_stormwater()),
        "continuous"  => Some(continuous_sim()),
        _ => None,
    }
}

/// Returns JSON array of {id, label, description} for the UI dropdown.
pub fn catalog_json() -> String {
    serde_json::to_string(&serde_json::json!([
        { "id": "detention",  "label": "Detention Sizing",       "description": "Rainfall → SCS → Unit Hydrograph → Detention Pond → Outfall" },
        { "id": "wq",         "label": "Water Quality Design",   "description": "Catchment → WQV → BMP Sizing → Treatment Train → Outfall" },
        { "id": "erosion",    "label": "Erosion & Sediment",     "description": "Catchment → Rational + RUSLE → Sediment Basin → Outfall" },
        { "id": "continuous", "label": "Continuous Simulation",  "description": "Site → 5-yr Hargreaves ET → Treatment Train → Outfall" },
        { "id": "full",       "label": "Full Stormwater Chain",  "description": "Complete hydrology → hydraulics → WQ design workflow" },
    ])).unwrap_or_default()
}
