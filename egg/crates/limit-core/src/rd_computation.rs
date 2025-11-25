// crates/limit-core/src/rd_computation.rs
//! Rate-Distortion computation with FGW (Fused Gromov-Wasserstein) distortion function

use serde::{Deserialize, Serialize};
use crate::{RDPoint, RDSeries};

/// FGW distortion parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FGWConfig {
    pub alpha: f64,  // Balance between feature and structure (0-1)
    pub epsilon: f64, // Entropic regularization
    pub max_iter: usize,
    pub tol: f64,
}

impl Default for FGWConfig {
    fn default() -> Self {
        Self {
            alpha: 0.5,
            epsilon: 0.01,
            max_iter: 100,
            tol: 1e-6,
        }
    }
}

/// RD computation engine
pub struct RDComputation {
    config: FGWConfig,
    series: RDSeries,
}

impl RDComputation {
    pub fn new(config: FGWConfig) -> Self {
        Self {
            config,
            series: RDSeries { points: Vec::new() },
        }
    }

    /// Compute FGW distortion between two distributions
    pub fn compute_fgw_distortion(
        &self,
        source_features: &[f64],
        target_features: &[f64],
        source_structure: &[Vec<f64>],
        target_structure: &[Vec<f64>],
    ) -> f64 {
        let feature_dist = self.compute_feature_distance(source_features, target_features);
        let structure_dist = self.compute_structure_distance(source_structure, target_structure);
        
        // FGW combines feature and structure distances
        self.config.alpha * feature_dist + (1.0 - self.config.alpha) * structure_dist
    }

    fn compute_feature_distance(&self, source: &[f64], target: &[f64]) -> f64 {
        source.iter()
            .zip(target.iter())
            .map(|(s, t)| (s - t).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    fn compute_structure_distance(&self, source: &[Vec<f64>], target: &[Vec<f64>]) -> f64 {
        let mut total_dist = 0.0;
        let n = source.len().min(target.len());
        
        for i in 0..n {
            for j in 0..n {
                let s_val = source.get(i).and_then(|row| row.get(j)).unwrap_or(&0.0);
                let t_val = target.get(i).and_then(|row| row.get(j)).unwrap_or(&0.0);
                total_dist += (s_val - t_val).powi(2);
            }
        }
        
        (total_dist / (n * n) as f64).sqrt()
    }

    /// Compute rate for given distortion using Shannon's rate-distortion theory
    pub fn compute_rate(&self, distortion: f64, variance: f64) -> f64 {
        if distortion >= variance {
            0.0
        } else {
            0.5 * (variance / distortion).ln() / std::f64::consts::LN_2
        }
    }

    /// Add RD point from refinement step
    pub fn add_refinement_point(&mut self, distortion: f64, variance: f64) {
        let rate = self.compute_rate(distortion, variance);
        let point = RDPoint {
            reward: rate,
            difficulty: distortion,
        };
        self.series.points.push(point);
    }

    /// Compute RD curve for multiple refinement steps
    pub fn compute_rd_curve(&mut self, refinement_steps: &[(f64, f64)]) -> &RDSeries {
        for (distortion, variance) in refinement_steps {
            self.add_refinement_point(*distortion, *variance);
        }
        &self.series
    }

    /// Find the "knee" point in RD curve (optimal operating point)
    pub fn find_knee_point(&self) -> Option<RDPoint> {
        if self.series.points.len() < 3 {
            return None;
        }

        let mut max_curvature = 0.0;
        let mut knee_idx = 0;

        for i in 1..self.series.points.len() - 1 {
            let p1 = &self.series.points[i - 1];
            let p2 = &self.series.points[i];
            let p3 = &self.series.points[i + 1];

            // Compute curvature using Menger curvature
            let curvature = self.compute_menger_curvature(
                (p1.difficulty, p1.reward),
                (p2.difficulty, p2.reward),
                (p3.difficulty, p3.reward),
            );

            if curvature > max_curvature {
                max_curvature = curvature;
                knee_idx = i;
            }
        }

        Some(self.series.points[knee_idx].clone())
    }

    fn compute_menger_curvature(&self, p1: (f64, f64), p2: (f64, f64), p3: (f64, f64)) -> f64 {
        let area = 0.5 * ((p2.0 - p1.0) * (p3.1 - p1.1) - (p3.0 - p1.0) * (p2.1 - p1.1)).abs();
        let d12 = ((p2.0 - p1.0).powi(2) + (p2.1 - p1.1).powi(2)).sqrt();
        let d23 = ((p3.0 - p2.0).powi(2) + (p3.1 - p2.1).powi(2)).sqrt();
        let d31 = ((p1.0 - p3.0).powi(2) + (p1.1 - p3.1).powi(2)).sqrt();

        if d12 * d23 * d31 == 0.0 {
            0.0
        } else {
            4.0 * area / (d12 * d23 * d31)
        }
    }

    /// Get the complete RD series
    pub fn get_series(&self) -> &RDSeries {
        &self.series
    }

    /// Compute distortion reduction from refinement
    pub fn compute_distortion_reduction(
        &self,
        original_features: &[f64],
        refined_features: &[f64],
        original_structure: &[Vec<f64>],
        refined_structure: &[Vec<f64>],
    ) -> f64 {
        let original_dist = self.compute_fgw_distortion(
            original_features,
            original_features,
            original_structure,
            original_structure,
        );
        
        let refined_dist = self.compute_fgw_distortion(
            original_features,
            refined_features,
            original_structure,
            refined_structure,
        );

        (original_dist - refined_dist).max(0.0)
    }

    /// Estimate variance from data
    pub fn estimate_variance(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fgw_distortion() {
        let config = FGWConfig::default();
        let rd = RDComputation::new(config);

        let source_features = vec![1.0, 2.0, 3.0];
        let target_features = vec![1.1, 2.1, 3.1];
        let source_structure = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let target_structure = vec![vec![0.0, 1.1], vec![1.1, 0.0]];

        let distortion = rd.compute_fgw_distortion(
            &source_features,
            &target_features,
            &source_structure,
            &target_structure,
        );

        assert!(distortion > 0.0);
    }

    #[test]
    fn test_rate_computation() {
        let config = FGWConfig::default();
        let rd = RDComputation::new(config);

        let rate = rd.compute_rate(0.5, 1.0);
        assert!(rate > 0.0);
    }

    #[test]
    fn test_knee_detection() {
        let config = FGWConfig::default();
        let mut rd = RDComputation::new(config);

        // Add points forming a curve
        rd.add_refinement_point(1.0, 2.0);
        rd.add_refinement_point(0.5, 2.0);
        rd.add_refinement_point(0.25, 2.0);
        rd.add_refinement_point(0.1, 2.0);

        let knee = rd.find_knee_point();
        assert!(knee.is_some());
    }
}
