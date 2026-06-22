class_name ActivityMapper
extends RefCounted
## Mappe les réponses santé daemon → intensité visuelle [0, 1].


static func from_health(status: String, llm_available: bool, embedding_available: bool) -> float:
	var base := 0.25
	if status == "ok":
		base = 0.55
	elif status == "degraded":
		base = 0.35
	if llm_available:
		base += 0.2
	if embedding_available:
		base += 0.15
	return clampf(base, 0.0, 1.0)


static func clamp_intensity(value: float) -> float:
	return clampf(value, 0.0, 1.0)


static func fallback_idle(elapsed: float) -> float:
	# Pulsation douce quand le daemon est hors ligne (mode dev).
	return clamp_intensity(0.2 + 0.1 * sin(elapsed * 1.5))