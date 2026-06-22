extends Node3D
## Fenêtre principale — Boule + layout responsive 1920×1080.

const MIN_WINDOW := Vector2i(1024, 600)

@onready var _daemon: TerritoryDaemonClient = $DaemonClient
@onready var _camera: Camera3D = $Camera3D


func _ready() -> void:
	_configure_window()
	WindowManager.register_main(self)
	if _daemon:
		_daemon.configure_window("main", ["chat", "memory", "graph", "monitoring"])
	var vp := get_viewport()
	if vp and not vp.size_changed.is_connected(_on_viewport_resized):
		vp.size_changed.connect(_on_viewport_resized)
	call_deferred("_on_viewport_resized")


func _configure_window() -> void:
	var win := get_window()
	if not win:
		return
	win.min_size = MIN_WINDOW
	if win.size.x < MIN_WINDOW.x or win.size.y < MIN_WINDOW.y:
		win.size = Vector2i(1920, 1080)


func _on_viewport_resized() -> void:
	if not _camera:
		return
	var size := get_viewport().get_visible_rect().size
	if size.y <= 0.0:
		return
	var aspect := size.x / size.y
	# Légère adaptation FOV pour garder la boule bien cadrée.
	_camera.fov = clampf(lerpf(58.0, 48.0, inverse_lerp(1.2, 2.2, aspect)), 44.0, 62.0)