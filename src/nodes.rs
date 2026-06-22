use serde::{Deserialize, Serialize};

pub const NODE_W: f64 = 180.0;  // wider to fit config/output text
pub const NODE_H: f64 = 84.0;      // total including header
pub const HEADER_H: f64 = 26.0;
pub const PORT_R: f64 = 6.0;
pub const PORT_HIT_R: f64 = 10.0;  // larger hit area

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    // ── Hydrology ──────────────────────────────────────────────────────────
    Catchment,
    RainfallEvent,
    RationalMethod,
    ScsRunoff,
    TimeOfConcentration,
    UnitHydrograph,
    LossMethod,
    ContinuousSim,
    // ── Hydraulics / Routing ───────────────────────────────────────────────
    ManningPipe,
    ManningChannel,
    GvfProfile,
    DetentionPond,
    PondChain,
    // ── Water Quality ──────────────────────────────────────────────────────
    WaterQualityVolume,
    BmpSizing,
    TreatmentBmp,
    TreatmentTrain,
    SedimentBasin,
    RusleErosion,
    // ── Sink ───────────────────────────────────────────────────────────────
    Outfall,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Hydrology,
    Hydraulics,
    WaterQuality,
    Sink,
}

impl Category {
    pub fn header_fill(self) -> &'static str {
        match self {
            Category::Hydrology    => "#1976d2",
            Category::Hydraulics   => "#e65100",
            Category::WaterQuality => "#2e7d32",
            Category::Sink         => "#546e7a",
        }
    }
    pub fn body_fill(self) -> &'static str {
        match self {
            Category::Hydrology    => "#e3f2fd",
            Category::Hydraulics   => "#fff3e0",
            Category::WaterQuality => "#e8f5e9",
            Category::Sink         => "#eceff1",
        }
    }
    pub fn border(self) -> &'static str {
        match self {
            Category::Hydrology    => "#1565c0",
            Category::Hydraulics   => "#bf360c",
            Category::WaterQuality => "#1b5e20",
            Category::Sink         => "#455a64",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Category::Hydrology    => "Hydrology",
            Category::Hydraulics   => "Hydraulics",
            Category::WaterQuality => "Water Quality",
            Category::Sink         => "Sink",
        }
    }
}

pub struct PortDef {
    pub label: &'static str,
}

/// A single user-editable parameter on a node.
pub struct ConfigFieldDef {
    pub key: &'static str,
    pub label: &'static str,
    /// "number" | "text" | "select"
    pub field_type: &'static str,
    pub default_num: f64,
    pub default_str: &'static str,
    pub unit: &'static str,
    /// (value, display_label) pairs — non-empty only for "select"
    pub options: &'static [(&'static str, &'static str)],
}

pub struct NodeDef {
    pub label: &'static str,
    pub category: Category,
    pub inputs: &'static [PortDef],
    pub outputs: &'static [PortDef],
    pub description: &'static str,
    /// Config fields — stored as owned Vec so no lifetime issue with inline arrays.
    pub config_fields: Vec<ConfigFieldDef>,
}

// ── Config field helper functions ─────────────────────────────────────────────

fn num(key: &'static str, label: &'static str, default: f64, unit: &'static str) -> ConfigFieldDef {
    ConfigFieldDef { key, label, field_type: "number", default_num: default, default_str: "", unit, options: &[] }
}
fn sel(key: &'static str, label: &'static str, default_str: &'static str, opts: &'static [(&'static str, &'static str)]) -> ConfigFieldDef {
    ConfigFieldDef { key, label, field_type: "select", default_num: 0.0, default_str, unit: "", options: opts }
}

// ── Option lists ─────────────────────────────────────────────────────────────

const LAND_USE_OPTS: &[(&str, &str)] = &[
    ("residential-low",    "Res. Low Density"),
    ("residential-medium", "Res. Medium Density"),
    ("residential-high",   "Res. High Density"),
    ("commercial",         "Commercial"),
    ("industrial",         "Industrial"),
    ("roadway",            "Roadway"),
    ("parking",            "Parking Lot"),
    ("open-space",         "Open Space / Park"),
    ("construction",       "Construction Site"),
    ("agricultural",       "Agricultural"),
];

const BMP_TYPE_OPTS: &[(&str, &str)] = &[
    ("bioretention",       "Bioretention / Rain Garden"),
    ("wet-pond",           "Wet Retention Pond"),
    ("constructed-wetland","Constructed Wetland"),
    ("vegetated-swale",    "Vegetated Swale"),
    ("sand-filter",        "Sand Filter"),
    ("infiltration-trench","Infiltration Trench"),
    ("permeable-pavement", "Permeable Pavement"),
    ("green-roof",         "Green Roof"),
    ("cistern",            "Cistern / RWH"),
];

const LOSS_METHOD_OPTS: &[(&str, &str)] = &[
    ("curve_number", "SCS Curve Number"),
    ("green_ampt",   "Green-Ampt"),
    ("horton",       "Horton"),
    ("init_const",   "Initial + Constant"),
    ("const_rate",   "Constant Rate"),
];

const HSG_OPTS: &[(&str, &str)] = &[
    ("A", "A — Well drained (sand)"),
    ("B", "B — Moderately drained"),
    ("C", "C — Poorly drained"),
    ("D", "D — Very poorly drained"),
];

const LOCATION_OPTS: &[(&str, &str)] = &[
    ("charlotte-nc",    "Charlotte NC"),
    ("raleigh-nc",      "Raleigh NC"),
    ("atlanta-ga",      "Atlanta GA"),
    ("new-york-ny",     "New York NY"),
    ("los-angeles-ca",  "Los Angeles CA"),
    ("chicago-il",      "Chicago IL"),
    ("dallas-tx",       "Dallas TX"),
    ("houston-tx",      "Houston TX"),
    ("washington-dc",   "Washington DC"),
    ("miami-fl",        "Miami FL"),
    ("philadelphia-pa", "Philadelphia PA"),
    ("phoenix-az",      "Phoenix AZ"),
    ("boston-ma",       "Boston MA"),
    ("detroit-mi",      "Detroit MI"),
    ("seattle-wa",      "Seattle WA"),
    ("minneapolis-mn",  "Minneapolis MN"),
    ("denver-co",       "Denver CO"),
];

impl NodeKind {
    pub fn def(self) -> NodeDef {
        match self {
            // ── Hydrology ─────────────────────────────────────────────────
            NodeKind::Catchment => NodeDef {
                label: "Catchment",
                category: Category::Hydrology,
                inputs: &[],
                outputs: &[PortDef { label: "Site data" }],
                description: "Drainage area + CN",
                config_fields: vec![
                    num("area_acres",   "Area",         5.0, "ac"),
                    num("curve_number", "Curve Number", 75.0, ""),
                    sel("land_use",     "Land Use",     "residential-medium", LAND_USE_OPTS),
                ],
            },
            NodeKind::RainfallEvent => NodeDef {
                label: "Rainfall Event",
                category: Category::Hydrology,
                inputs: &[],
                outputs: &[PortDef { label: "P (in)" }],
                description: "Storm depth + duration",
                config_fields: vec![
                    num("depth_in",    "Rainfall depth", 3.5, "in"),
                    num("duration_hr", "Duration",       24.0, "hr"),
                ],
            },
            NodeKind::RationalMethod => NodeDef {
                label: "Rational Method",
                category: Category::Hydrology,
                inputs: &[
                    PortDef { label: "Catchment" },
                    PortDef { label: "IDF curve" },
                ],
                outputs: &[PortDef { label: "Q (cfs)" }],
                description: "Q = C·i·A",
                config_fields: vec![
                    num("area_acres",      "Area",      5.0, "ac"),
                    num("runoff_c",        "Runoff C",  0.70, ""),
                    num("intensity_in_hr", "Intensity", 2.5,  "in/hr"),
                ],
            },
            NodeKind::ScsRunoff => NodeDef {
                label: "SCS CN Runoff",
                category: Category::Hydrology,
                inputs: &[
                    PortDef { label: "Catchment" },
                    PortDef { label: "Rainfall" },
                ],
                outputs: &[PortDef { label: "Q (in)" }],
                description: "Curve-number runoff depth",
                config_fields: vec![
                    num("curve_number", "Curve Number", 75.0, ""),
                    num("rainfall_in",  "Rainfall",     3.5,  "in"),
                    num("area_acres",   "Area",         5.0,  "ac"),
                ],
            },
            NodeKind::TimeOfConcentration => NodeDef {
                label: "Time of Conc.",
                category: Category::Hydrology,
                inputs: &[PortDef { label: "Flow path" }],
                outputs: &[PortDef { label: "Tc (min)" }],
                description: "TR-55 segmented Tc",
                config_fields: vec![
                    num("tc_min", "Tc (override)", 20.0, "min"),
                ],
            },
            NodeKind::UnitHydrograph => NodeDef {
                label: "Unit Hydrograph",
                category: Category::Hydrology,
                inputs: &[
                    PortDef { label: "Q runoff" },
                    PortDef { label: "Tc (min)" },
                ],
                outputs: &[PortDef { label: "Peak Q (cfs)" }],
                description: "SCS triangular UH",
                config_fields: vec![
                    num("area_acres", "Area", 5.0, "ac"),
                    num("tc_min",     "Tc",   20.0, "min"),
                ],
            },
            NodeKind::LossMethod => NodeDef {
                label: "Loss Method",
                category: Category::Hydrology,
                inputs: &[PortDef { label: "Rainfall" }],
                outputs: &[PortDef { label: "Excess (in)" }],
                description: "Green-Ampt / Horton / CN",
                config_fields: vec![
                    sel("method",       "Method",       "curve_number", LOSS_METHOD_OPTS),
                    num("curve_number", "Curve Number", 75.0, ""),
                    sel("hsg",          "Soil Group",   "B",  HSG_OPTS),
                    num("rainfall_in",  "Rainfall",     3.5,  "in"),
                    num("duration_hr",  "Duration",     24.0, "hr"),
                ],
            },
            NodeKind::ContinuousSim => NodeDef {
                label: "Continuous Sim",
                category: Category::Hydrology,
                inputs: &[PortDef { label: "Site" }],
                outputs: &[PortDef { label: "Q (ac-in/yr)" }],
                description: "Multi-year Hargreaves ET",
                config_fields: vec![
                    sel("location",     "City",         "charlotte-nc",        LOCATION_OPTS),
                    num("area_acres",   "Area",         5.0,  "ac"),
                    num("curve_number", "Curve Number", 75.0, ""),
                    sel("land_use",     "Land Use",     "residential-medium", LAND_USE_OPTS),
                    num("years",        "Years",        3.0,  "yr"),
                ],
            },
            // ── Hydraulics ────────────────────────────────────────────────
            NodeKind::ManningPipe => NodeDef {
                label: "Manning Pipe",
                category: Category::Hydraulics,
                inputs: &[PortDef { label: "Q design (cfs)" }],
                outputs: &[
                    PortDef { label: "Q full (cfs)" },
                    PortDef { label: "Depth (ft)" },
                ],
                description: "Circular pipe capacity",
                config_fields: vec![
                    num("diameter_ft",   "Diameter",    1.5,   "ft"),
                    num("slope",         "Slope",       0.005, "ft/ft"),
                    num("manning_n",     "Manning n",   0.013, ""),
                    num("design_q_cfs",  "Design Q",    5.0,   "cfs"),
                ],
            },
            NodeKind::ManningChannel => NodeDef {
                label: "Manning Channel",
                category: Category::Hydraulics,
                inputs: &[PortDef { label: "Q (cfs)" }],
                outputs: &[
                    PortDef { label: "Depth (ft)" },
                    PortDef { label: "V (fps)" },
                ],
                description: "Trapezoidal normal depth",
                config_fields: vec![
                    num("bottom_width_ft", "Bottom width", 4.0,   "ft"),
                    num("side_slope",      "Side slope",   3.0,   "H:V"),
                    num("slope",           "Slope",        0.01,  "ft/ft"),
                    num("manning_n",       "Manning n",    0.035, ""),
                    num("design_q_cfs",    "Design Q",     5.0,   "cfs"),
                ],
            },
            NodeKind::GvfProfile => NodeDef {
                label: "GVF Profile",
                category: Category::Hydraulics,
                inputs: &[
                    PortDef { label: "Q (cfs)" },
                    PortDef { label: "Tailwater (ft)" },
                ],
                outputs: &[PortDef { label: "WSP depth (ft)" }],
                description: "Standard Step GVF",
                config_fields: vec![
                    num("design_q_cfs",    "Design Q",    5.0,   "cfs"),
                    num("bottom_width_ft", "Bottom width",4.0,   "ft"),
                    num("side_slope",      "Side slope",  3.0,   "H:V"),
                    num("slope",           "Bed slope",   0.01,  "ft/ft"),
                    num("manning_n",       "Manning n",   0.035, ""),
                    num("tailwater_ft",    "Tailwater",   0.0,   "ft"),
                ],
            },
            NodeKind::DetentionPond => NodeDef {
                label: "Detention Pond",
                category: Category::Hydraulics,
                inputs: &[PortDef { label: "Inflow hydrograph" }],
                outputs: &[
                    PortDef { label: "Peak Q out (cfs)" },
                    PortDef { label: "Storage (cf)" },
                ],
                description: "Modified Puls routing",
                config_fields: vec![
                    num("area_acres",       "Drainage area",    5.0,   "ac"),
                    num("curve_number",     "Curve Number",     80.0,  ""),
                    num("rainfall_in",      "Design storm",     4.2,   "in"),
                    num("tc_min",           "Tc",               20.0,  "min"),
                    num("bottom_area_sf",   "Pond bottom area", 5000.0,"sf"),
                    num("side_slope",       "Side slope",       3.0,   "H:V"),
                    num("max_depth_ft",     "Max depth",        6.0,   "ft"),
                    num("orifice_dia_in",   "Orifice dia.",     6.0,   "in"),
                    num("orifice_invert_ft","Orifice invert",   0.0,   "ft"),
                ],
            },
            NodeKind::PondChain => NodeDef {
                label: "Pond Chain",
                category: Category::Hydraulics,
                inputs: &[PortDef { label: "Inflow" }],
                outputs: &[PortDef { label: "Peak Q out (cfs)" }],
                description: "Series pond routing",
                config_fields: vec![
                    num("area_acres",    "Drainage area", 5.0,  "ac"),
                    num("curve_number",  "Curve Number",  80.0, ""),
                    num("rainfall_in",   "Design storm",  4.2,  "in"),
                ],
            },
            // ── Water Quality ─────────────────────────────────────────────
            NodeKind::WaterQualityVolume => NodeDef {
                label: "WQ Volume",
                category: Category::WaterQuality,
                inputs: &[
                    PortDef { label: "Catchment" },
                    PortDef { label: "Design storm" },
                ],
                outputs: &[PortDef { label: "WQV (cf)" }],
                description: "WQV = Rv·P·A·3630",
                config_fields: vec![
                    num("area_acres",      "Area",         5.0, "ac"),
                    num("impervious_pct",  "Impervious",   50.0,"%"),
                    num("design_storm_in", "Design storm", 1.0, "in"),
                ],
            },
            NodeKind::BmpSizing => NodeDef {
                label: "BMP Sizing",
                category: Category::WaterQuality,
                inputs: &[PortDef { label: "WQV (cf)" }],
                outputs: &[PortDef { label: "Surface area (sf)" }],
                description: "Area from treated volume",
                config_fields: vec![
                    sel("bmp_type",        "BMP type",     "bioretention",    BMP_TYPE_OPTS),
                    num("area_acres",      "Area",         5.0, "ac"),
                    num("impervious_pct",  "Impervious",   50.0,"%"),
                    num("design_storm_in", "Design storm", 1.0, "in"),
                ],
            },
            NodeKind::TreatmentBmp => NodeDef {
                label: "Treatment BMP",
                category: Category::WaterQuality,
                inputs: &[PortDef { label: "Influent loads" }],
                outputs: &[PortDef { label: "Effluent loads" }],
                description: "Single BMP mass balance",
                config_fields: vec![
                    sel("bmp_type",  "BMP type",  "bioretention",       BMP_TYPE_OPTS),
                    num("area_acres","Area",       5.0, "ac"),
                    sel("land_use",  "Land Use",  "residential-medium", LAND_USE_OPTS),
                    num("runoff_in", "Runoff",    0.5, "in"),
                ],
            },
            NodeKind::TreatmentTrain => NodeDef {
                label: "Treatment Train",
                category: Category::WaterQuality,
                inputs: &[PortDef { label: "Influent loads" }],
                outputs: &[
                    PortDef { label: "Effluent loads" },
                    PortDef { label: "TSS η (%)" },
                ],
                description: "Series BMPs in chain",
                config_fields: vec![
                    num("area_acres", "Area",     5.0, "ac"),
                    sel("land_use",   "Land Use", "residential-medium", LAND_USE_OPTS),
                    num("runoff_in",  "Runoff",   0.5, "in"),
                    // bmp_chain is a special comma-separated field handled in executor
                ],
            },
            NodeKind::SedimentBasin => NodeDef {
                label: "Sediment Basin",
                category: Category::WaterQuality,
                inputs: &[
                    PortDef { label: "Q design (cfs)" },
                    PortDef { label: "Soil loss (t/ac/yr)" },
                ],
                outputs: &[
                    PortDef { label: "Trap η (%)" },
                    PortDef { label: "Volume (cf)" },
                ],
                description: "NCDEQ 435 sf/cfs method",
                config_fields: vec![
                    num("area_acres",            "Area",        5.0,  "ac"),
                    num("design_q_cfs",          "Design Q",    5.0,  "cfs"),
                    num("sed_yield_tons_ac_yr",  "Sed. yield",  10.0, "t/ac/yr"),
                ],
            },
            NodeKind::RusleErosion => NodeDef {
                label: "RUSLE Erosion",
                category: Category::WaterQuality,
                inputs: &[
                    PortDef { label: "Catchment" },
                    PortDef { label: "Soil/slope" },
                ],
                outputs: &[PortDef { label: "A (t/ac/yr)" }],
                description: "A = R·K·LS·C·P",
                config_fields: vec![
                    sel("region",      "Region",       "charlotte-nc",       LOCATION_OPTS),
                    num("area_acres",  "Area",         1.0,   "ac"),
                    num("slope_length_ft","Slope length",100.0,"ft"),
                    num("slope_pct",   "Slope",        5.0,   "%"),
                ],
            },
            // ── Sink ──────────────────────────────────────────────────────
            NodeKind::Outfall => NodeDef {
                label: "Outfall",
                category: Category::Sink,
                inputs: &[PortDef { label: "Flow / loads" }],
                outputs: &[],
                description: "System discharge point",
                config_fields: Vec::new(),
            },
        }
    }

    pub fn all() -> &'static [NodeKind] {
        &[
            NodeKind::Catchment,
            NodeKind::RainfallEvent,
            NodeKind::RationalMethod,
            NodeKind::ScsRunoff,
            NodeKind::TimeOfConcentration,
            NodeKind::UnitHydrograph,
            NodeKind::LossMethod,
            NodeKind::ContinuousSim,
            NodeKind::ManningPipe,
            NodeKind::ManningChannel,
            NodeKind::GvfProfile,
            NodeKind::DetentionPond,
            NodeKind::PondChain,
            NodeKind::WaterQualityVolume,
            NodeKind::BmpSizing,
            NodeKind::TreatmentBmp,
            NodeKind::TreatmentTrain,
            NodeKind::SedimentBasin,
            NodeKind::RusleErosion,
            NodeKind::Outfall,
        ]
    }

    /// Returns (world_x, world_y) of a port relative to node position.
    pub fn port_pos(self, port_idx: usize, is_output: bool) -> (f64, f64) {
        let def = self.def();
        let n = if is_output { def.outputs.len() } else { def.inputs.len() };
        let body_h = NODE_H - HEADER_H;
        let y_offset = HEADER_H + body_h * (port_idx as f64 + 1.0) / (n as f64 + 1.0);
        let x_offset = if is_output { NODE_W } else { 0.0 };
        (x_offset, y_offset)
    }
}

/// Absolute port position given a node's (x, y).
pub fn abs_port_pos(kind: NodeKind, node_x: f64, node_y: f64, port_idx: usize, is_output: bool) -> (f64, f64) {
    let (dx, dy) = kind.port_pos(port_idx, is_output);
    (node_x + dx, node_y + dy)
}
