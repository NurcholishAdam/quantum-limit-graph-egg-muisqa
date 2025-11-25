// crates/limit-core/src/types.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TraceId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub source: String,           // e.g., lecture path or KG node id
    pub operation: String,        // add/merge/split/remove/rewire
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceCheckpoint {
    pub label: String,            // e.g., "no-jailbreak-merge"
    pub passed: bool,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RDPoint {
    pub step: u32,
    pub rate: f32,
    pub distortion: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RDSeries {
    pub points: Vec<RDPoint>,
}

impl RDSeries {
    pub fn add(&mut self, p: RDPoint) { self.points.push(p); }
    pub fn knee_index(&self) -> Option<usize> {
        if self.points.len() < 3 { return None; }
        // Simple geometric knee heuristic: max perpendicular distance to line(first,last)
        let first = &self.points[0];
        let last = &self.points[self.points.len()-1];
        let ax = last.distortion - first.distortion;
        let ay = last.rate - first.rate;

        let mut best: Option<(usize, f32)> = None;
        for (i, p) in self.points.iter().enumerate().skip(1).take(self.points.len()-2) {
            let vx = p.distortion - first.distortion;
            let vy = p.rate - first.rate;
            // cross product magnitude / line length
            let cross = (ax*vy - ay*vx).abs();
            let len = (ax.powi(2) + ay.powi(2)).sqrt();
            let dist = cross / len.max(1e-6);
            if best.map_or(true, |(_, b)| dist > b) { best = Some((i, dist)); }
        }
        best.map(|(i, _)| i)
    }
}
