class_name VisualEventMapper
extends RefCounted
## Mappe les événements backend WS → effets visuels boule (Phase 19).

const THROTTLE_SECS := 0.1

static var _last_applied := 0.0


static func from_backend_event(event: String, payload: Dictionary) -> Dictionary:
	match event:
		"memory_assimilated":
			return {
				"kind": "assimilation",
				"pulse_boost": 0.75,
				"pulse_duration": 0.85,
				"swirl": true,
				"stress": 0.0,
				"flash_rotation": 0.15,
			}
		"brain_pulse":
			return {
				"kind": str(payload.get("kind", "pulse")),
				"pulse_boost": float(payload.get("boost", 0.45)),
				"pulse_duration": float(payload.get("duration", 0.5)),
				"swirl": false,
				"stress": 0.0,
				"flash_rotation": 0.08,
			}
		"tool_call":
			return {
				"kind": "tool_call",
				"pulse_boost": 0.55,
				"pulse_duration": 0.4,
				"swirl": false,
				"stress": 0.0,
				"flash_rotation": 0.22,
			}
		"vector_search":
			return {
				"kind": "search",
				"pulse_boost": 0.35,
				"pulse_duration": 0.45,
				"swirl": true,
				"stress": 0.0,
				"flash_rotation": 0.05,
			}
		"system_error":
			return {
				"kind": "error",
				"pulse_boost": 0.25,
				"pulse_duration": 0.35,
				"swirl": false,
				"stress": 0.85,
				"flash_rotation": 0.0,
				"shake": true,
			}
		"degraded_mode":
			return {
				"kind": "degraded",
				"pulse_boost": 0.15,
				"pulse_duration": 0.3,
				"swirl": false,
				"stress": 0.55,
				"flash_rotation": 0.0,
			}
		_:
			return {}


static func should_apply(effect: Dictionary) -> bool:
	if effect.is_empty():
		return false
	var now := Time.get_ticks_msec() / 1000.0
	if now - _last_applied < THROTTLE_SECS:
		var kind: String = str(effect.get("kind", ""))
		if kind not in ["assimilation", "error", "degraded"]:
			return false
	_last_applied = now
	return true


static func idle_breathing(elapsed: float, connected: bool) -> Dictionary:
	if connected:
		return {}
	return {
		"kind": "idle",
		"pulse_boost": 0.08 + 0.04 * sin(elapsed * 1.2),
		"pulse_duration": 0.2,
		"stress": 0.25,
		"swirl": false,
	}