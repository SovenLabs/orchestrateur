extends Node3D
class_name AgentSphere
## Sphère 3D représentant un agent persistant (Phase 5).

@export var agent_id: String = ""
@export var display_name: String = ""
@export var agent_role: String = "assistant"
@export var agent_status: String = "sleeping"

var _mesh: MeshInstance3D
var _pulse := 0.0


func _ready() -> void:
	_mesh = MeshInstance3D.new()
	var sphere := SphereMesh.new()
	sphere.radius = 0.35
	sphere.height = 0.7
	_mesh.mesh = sphere
	var mat := StandardMaterial3D.new()
	mat.albedo_color = _color_for_status(agent_status)
	mat.emission_enabled = true
	mat.emission = mat.albedo_color * 0.35
	_mesh.material_override = mat
	add_child(_mesh)


func configure(id: String, name: String, role: String, status: String) -> void:
	agent_id = id
	display_name = name
	agent_role = role
	set_status(status)


func set_status(status: String) -> void:
	agent_status = status
	if _mesh and _mesh.material_override is StandardMaterial3D:
		var mat := _mesh.material_override as StandardMaterial3D
		mat.albedo_color = _color_for_status(status)
		mat.emission = mat.albedo_color * (0.55 if status == "awake" else 0.25)


func pulse_on_message() -> void:
	_pulse = 1.0
	if _mesh and _mesh.material_override is StandardMaterial3D:
		var mat := _mesh.material_override as StandardMaterial3D
		mat.emission_energy_multiplier = 3.2


func _process(delta: float) -> void:
	if _pulse > 0.0:
		_pulse = maxf(0.0, _pulse - delta * 2.2)
		var s := 1.0 + _pulse * 0.42
		scale = Vector3.ONE * s
		if _mesh and _mesh.material_override is StandardMaterial3D:
			var mat := _mesh.material_override as StandardMaterial3D
			mat.emission_energy_multiplier = lerpf(0.55, 3.2, _pulse)
	else:
		scale = Vector3.ONE
		if _mesh and _mesh.material_override is StandardMaterial3D:
			var mat := _mesh.material_override as StandardMaterial3D
			mat.emission_energy_multiplier = 0.55 if agent_status == "awake" else 0.25


func _color_for_status(status: String) -> Color:
	match status:
		"awake":
			return Color(0.55, 0.85, 0.95)
		"background":
			return Color(0.95, 0.75, 0.35)
		_:
			return Color(0.45, 0.55, 0.72)