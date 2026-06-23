extends Node3D
## Scène légère pour embed HTML5/WASM — agents orbitaux sans dock UI.

@onready var _daemon: TerritoryDaemonClient = $DaemonClient
@onready var _camera: Camera3D = $Camera3D


func _ready() -> void:
	if _daemon:
		_daemon.configure_window("embed", [])
	_camera.make_current()


func _process(delta: float) -> void:
	_camera.position = Vector3(
		sin(Time.get_ticks_msec() * 0.00015) * 1.2,
		0.55 + sin(Time.get_ticks_msec() * 0.00022) * 0.15,
		9.5 + cos(Time.get_ticks_msec() * 0.00012) * 0.4,
	)
	_camera.look_at(Vector3.ZERO, Vector3.UP)