//! Métriques frame time — rolling window pour diagnostic performance HUD.

const SAMPLE_CAP: usize = 120;

/// Statistiques glissantes de durée de frame (ms).
#[derive(Debug, Clone, Default)]
pub struct FrameMetrics {
    samples: Vec<f32>,
    index: usize,
}

impl FrameMetrics {
    /// Enregistre une mesure en millisecondes.
    pub fn record(&mut self, ms: f32) {
        if self.samples.len() < SAMPLE_CAP {
            self.samples.push(ms);
        } else {
            self.samples[self.index] = ms;
            self.index = (self.index + 1) % SAMPLE_CAP;
        }
    }

    /// Dernière mesure enregistrée.
    #[must_use]
    pub fn last(&self) -> Option<f32> {
        if self.samples.is_empty() {
            None
        } else if self.samples.len() < SAMPLE_CAP {
            self.samples.last().copied()
        } else {
            let idx = self.index.checked_sub(1).unwrap_or(SAMPLE_CAP - 1);
            Some(self.samples[idx])
        }
    }

    /// Moyenne sur la fenêtre courante.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn average(&self) -> Option<f32> {
        if self.samples.is_empty() {
            return None;
        }
        let len = self.samples.len();
        let sum: f32 = self.samples.iter().sum();
        Some(sum / len as f32)
    }

    /// Percentile 99 approximatif (tri local, fenêtre ≤ 120).
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn p99(&self) -> Option<f32> {
        if self.samples.is_empty() {
            return None;
        }
        let mut sorted = self.samples.clone();
        sorted.sort_by(f32::total_cmp);
        let len = sorted.len();
        let idx = ((len as f32) * 0.99).floor() as usize;
        let idx = idx.min(len.saturating_sub(1));
        Some(sorted[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_averages() {
        let mut m = FrameMetrics::default();
        m.record(4.0);
        m.record(6.0);
        assert!((m.average().unwrap() - 5.0).abs() < f32::EPSILON);
    }
}