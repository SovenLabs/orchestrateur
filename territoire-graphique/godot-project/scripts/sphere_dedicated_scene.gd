extends Node3D
## Fenêtre dédiée — Boule de Pixels Vivante premium (Phase 24).

const BRAIN_ENV := preload("res://scripts/ai_neural_brain/neural_brain_environment.gd")

@onready var _world_env: WorldEnvironment = $WorldEnvironment
@onready var _brain: AINeuralBrainSphere = $AINeuralBrainSphere
@onready var _daemon: TerritoryDaemonClient = $DaemonClient
@onready var _fps_label: Label = $UI/FpsLabel
@onready var _perf: Node = $SpherePerformance


func _ready() -> void:
	_configure_window()
	_setup_environment()
	if _daemon:
		_daemon.configure_window("sphere", ["sphere"])
		_daemon.brain_pulse_requested.connect(_on_brain_pulse)
	if _perf and _perf.has_signal("fps_updated"):
		_perf.fps_updated.connect(_on_fps)


func _setup_environment() -> void:
	if _world_env == null:
		return
	var env_script: Script = BRAIN_ENV
	_world_env.set_script(env_script)
	if _world_env.has_method("apply_cinematic_glow"):
		_world_env.apply_cinematic_glow()
	_world_env.add_to_group("territory_environment")


func _configure_window() -> void:
	var win := get_window()
	if win:
		win.title = "Orchestrateur — Boule de Pixels Vivante"
		win.size = Vector2i(1280, 800)
		win.min_size = Vector2i(800, 600)


func _on_fps(fps: float, tier: String) -> void:
	if _fps_label:
		_fps_label.text = "Sphère — %.0f FPS · %s" % [fps, tier]


func _on_brain_pulse(boost: float, duration: float) -> void:
	if _brain:
		_brain.pulse_activity(boost, duration)