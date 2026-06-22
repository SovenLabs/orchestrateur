class_name NeuralBrainCamera
extends Camera3D
## Orbite souris + zoom molette + rotation auto optionnelle.

@export_category("Orbite")
@export var orbit_enabled := true
@export var min_distance := 5.0
@export var max_distance := 18.0
@export var zoom_speed := 0.8
@export var rotate_sensitivity := 0.004

@export_category("Rotation auto")
@export var auto_rotate := true
@export var auto_rotate_speed := 0.12

@export var target_path: NodePath = ^".."

var _target: Node3D
var _yaw := 0.55
var _pitch := 0.18
var _distance := 9.5
var _dragging := false


func _ready() -> void:
	_target = get_node_or_null(target_path) as Node3D
	if _target == null:
		_target = get_parent() as Node3D
	_reposition()


func _unhandled_input(event: InputEvent) -> void:
	if not orbit_enabled:
		return
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index == MOUSE_BUTTON_LEFT:
			_dragging = mb.pressed
		elif mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
			_distance = maxf(min_distance, _distance - zoom_speed)
			_reposition()
		elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
			_distance = minf(max_distance, _distance + zoom_speed)
			_reposition()
	elif event is InputEventMouseMotion and _dragging:
		var mm := event as InputEventMouseMotion
		_yaw -= mm.relative.x * rotate_sensitivity
		_pitch = clampf(_pitch - mm.relative.y * rotate_sensitivity, -0.35, 0.85)
		_reposition()


func _process(delta: float) -> void:
	if auto_rotate and not _dragging:
		_yaw += auto_rotate_speed * delta
		_reposition()


func set_auto_rotate(enabled: bool) -> void:
	auto_rotate = enabled


func _reposition() -> void:
	if _target == null:
		return
	var offset := Vector3(
		sin(_yaw) * cos(_pitch),
		sin(_pitch),
		cos(_yaw) * cos(_pitch),
	) * _distance
	global_position = _target.global_position + offset
	look_at(_target.global_position, Vector3.UP)