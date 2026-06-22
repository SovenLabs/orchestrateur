extends Node
## Mapper central événements backend → effets visuels (Phase 20).
## Autoload : /root/VisualEventMapper — pas de class_name (évite conflit autoload).

signal visual_effect_triggered(effect_name: String, intensity: float)
signal effect_ready(effect: Dictionary)
signal activity_level_changed(level: float)

const THROTTLE_SECS := 1.0 / 30.0

var activity_level: float = 0.6

var _last_applied := 0.0
var _degraded := false


func map_backend_event(event_type: String, payload: Dictionary = {}) -> void:
	var effect := _build_effect(event_type, payload)
	if effect.is_empty():
		return
	if not _should_apply(effect):
		return

	var effect_name := str(effect.get("kind", event_type))
	var intensity := float(effect.get("intensity", effect.get("pulse_boost", 0.5)))
	if effect_name == "activity_change":
		activity_level_changed.emit(activity_level)
	visual_effect_triggered.emit(effect_name, intensity)
	effect_ready.emit(effect)


func update_global_activity(level: float) -> void:
	activity_level = clampf(level, 0.0, 3.0)
	activity_level_changed.emit(activity_level)
	visual_effect_triggered.emit("activity_change", activity_level)


func set_degraded_mode(active: bool) -> void:
	if _degraded == active:
		return
	_degraded = active
	if active:
		map_backend_event("degraded_mode", {"message": "Mode dégradé"})
	else:
		visual_effect_triggered.emit("recovery", 0.4)
		effect_ready.emit({
			"kind": "recovery",
			"stress": 0.0,
			"swirl": false,
			"particle_mode": "idle",
		})


func emit_idle_breathing(elapsed: float, connected: bool) -> void:
	if connected:
		return
	var breath := 0.08 + 0.04 * sin(elapsed * 1.2)
	var effect := {
		"kind": "idle",
		"intensity": breath,
		"pulse_boost": breath,
		"pulse_duration": 0.2,
		"stress": 0.25,
		"swirl": false,
		"particle_mode": "idle",
	}
	if _should_apply(effect):
		visual_effect_triggered.emit("idle", breath)
		effect_ready.emit(effect)


func should_apply(effect: Dictionary) -> bool:
	return _should_apply(effect)


func _should_apply(effect: Dictionary) -> bool:
	if effect.is_empty():
		return false
	var now := Time.get_ticks_msec() / 1000.0
	if now - _last_applied < THROTTLE_SECS:
		var kind: String = str(effect.get("kind", ""))
		if kind not in ["assimilation", "error", "degraded", "tool_call"]:
			return false
	_last_applied = now
	return true


func _build_effect(event_type: String, payload: Dictionary) -> Dictionary:
	match event_type:
		"memory_assimilated":
			var intensity := clampf(float(payload.get("intensity", 1.0)), 0.4, 1.0)
			return {
				"kind": "assimilation",
				"intensity": intensity,
				"pulse_boost": 0.75 * intensity,
				"pulse_duration": 0.85,
				"swirl": true,
				"stress": 0.0,
				"flash_rotation": 0.15,
				"particle_mode": "assimilation",
			}
		"brain_pulse":
			var boost := float(payload.get("boost", 0.45))
			return {
				"kind": str(payload.get("kind", "pulse")),
				"intensity": boost,
				"pulse_boost": boost,
				"pulse_duration": float(payload.get("duration", 0.5)),
				"swirl": false,
				"stress": 0.0,
				"flash_rotation": 0.08,
				"particle_mode": "idle",
			}
		"tool_call":
			return {
				"kind": "tool_call",
				"intensity": 0.8,
				"pulse_boost": 0.55,
				"pulse_duration": 0.4,
				"swirl": false,
				"stress": 0.0,
				"flash_rotation": 0.22,
				"particle_mode": "tool_call",
				"tool_name": str(payload.get("tool_name", "")),
			}
		"vector_search":
			return {
				"kind": "search",
				"intensity": 0.55,
				"pulse_boost": 0.35,
				"pulse_duration": 0.45,
				"swirl": true,
				"stress": 0.0,
				"flash_rotation": 0.05,
				"particle_mode": "idle",
			}
		"graph_changed", "graph_updated":
			return {
				"kind": "graph_update",
				"intensity": 0.45,
				"pulse_boost": 0.3,
				"pulse_duration": 0.5,
				"swirl": true,
				"stress": 0.0,
				"flash_rotation": 0.06,
				"particle_mode": "idle",
			}
		"agent_activity":
			var level := clampf(float(payload.get("level", 0.5)), 0.0, 3.0)
			activity_level = level
			return {
				"kind": "activity_change",
				"intensity": level / 3.0,
				"pulse_boost": 0.0,
				"pulse_duration": 0.0,
				"swirl": false,
				"stress": 0.0,
				"particle_mode": "idle",
			}
		"system_error", "error_occurred":
			return {
				"kind": "error",
				"intensity": 1.0,
				"pulse_boost": 0.25,
				"pulse_duration": 0.35,
				"swirl": false,
				"stress": 0.85,
				"flash_rotation": 0.0,
				"shake": true,
				"particle_mode": "tool_call",
			}
		"degraded_mode":
			return {
				"kind": "degraded",
				"intensity": 0.6,
				"pulse_boost": 0.15,
				"pulse_duration": 0.3,
				"swirl": false,
				"stress": 0.55,
				"flash_rotation": 0.0,
				"particle_mode": "idle",
			}
		_:
			return {}