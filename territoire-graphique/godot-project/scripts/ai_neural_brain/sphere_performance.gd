extends Node
## Surveillance FPS + mode adaptatif (Phase 24) — cible 60 FPS stable.

signal fps_updated(fps: float, tier: String)
signal quality_changed(tier: String)

const TARGET_FPS := 60.0
const WARN_FPS := 50.0
const CRITICAL_FPS := 42.0

@export var brain_path: NodePath
@export var report_interval_secs := 0.5

var current_fps := 60.0
var quality_tier := "high"

var _brain: AINeuralBrainSphere
var _elapsed := 0.0


func _ready() -> void:
	_brain = get_node_or_null(brain_path) as AINeuralBrainSphere


func _process(delta: float) -> void:
	_elapsed += delta
	if _elapsed < report_interval_secs:
		return
	_elapsed = 0.0

	current_fps = Engine.get_frames_per_second()
	var tier := _resolve_tier(current_fps)
	if tier != quality_tier:
		quality_tier = tier
		_apply_tier(tier)
		quality_changed.emit(tier)
	fps_updated.emit(current_fps, tier)


func _resolve_tier(fps: float) -> String:
	if fps >= TARGET_FPS - 2.0:
		return "high"
	if fps >= WARN_FPS:
		return "medium"
	if fps >= CRITICAL_FPS:
		return "low"
	return "critical"


func _apply_tier(tier: String) -> void:
	if _brain == null:
		return
	match tier:
		"high":
			_brain.set_performance_tier(1.0)
		"medium":
			_brain.set_performance_tier(0.75)
		"low":
			_brain.set_performance_tier(0.55)
		"critical":
			_brain.set_performance_tier(0.35)