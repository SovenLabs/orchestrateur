extends Node3D
## Boule de Pixels Vivante — shader + particules réactives à l'activité daemon.

@export var rotation_speed := 0.35

@onready var _mesh: MeshInstance3D = $Mesh
@onready var _particles: GPUParticles3D = $Particles

var activity_intensity := 0.35
var _material: ShaderMaterial


func _ready() -> void:
	_material = _mesh.material_override as ShaderMaterial
	if DaemonClient:
		DaemonClient.activity_changed.connect(update_brain_activity)
		update_brain_activity(activity_intensity)


func _process(delta: float) -> void:
	rotate_y(rotation_speed * delta * (0.5 + activity_intensity))
	if _material:
		_material.set_shader_parameter("time", Time.get_ticks_msec() / 1000.0)
		_material.set_shader_parameter("activity", activity_intensity)
	_update_particles()


func update_brain_activity(intensity: float) -> void:
	activity_intensity = clampf(intensity, 0.0, 1.0)
	if _material:
		_material.set_shader_parameter("activity", activity_intensity)
	_update_particles()


func _update_particles() -> void:
	if not _particles:
		return
	var amount := int(lerpf(24.0, 220.0, activity_intensity))
	_particles.amount = amount
	var process_mat: ParticleProcessMaterial = _particles.process_material
	if process_mat:
		process_mat.emission_sphere_radius = lerpf(1.05, 1.35, activity_intensity)
		process_mat.initial_velocity_min = lerpf(0.2, 1.2, activity_intensity)
		process_mat.initial_velocity_max = lerpf(0.8, 2.5, activity_intensity)
	_particles.speed_scale = lerpf(0.6, 1.8, activity_intensity)