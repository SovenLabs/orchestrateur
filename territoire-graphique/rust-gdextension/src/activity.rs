//! Mapping santé daemon → intensité visuelle (miroir de `activity_mapper.gd`).

/// Calcule l'intensité [0, 1] à partir d'une réponse Health.
#[must_use]
pub fn map_health_to_activity(status: &str, llm_available: bool, embedding_available: bool) -> f32 {
    let mut base: f32 = match status {
        "ok" => 0.55,
        "degraded" => 0.35,
        _ => 0.25,
    };
    if llm_available {
        base += 0.2;
    }
    if embedding_available {
        base += 0.15;
    }
    base.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_health_is_high() {
        let v = map_health_to_activity("ok", true, true);
        assert!(v >= 0.85);
    }
}