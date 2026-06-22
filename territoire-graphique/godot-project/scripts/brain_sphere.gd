extends Node3D
## Boule de Pixels Vivante v2 — particules seuillées + lissage d'activité.

@export var rotation_speed := 0.35
@export var particle_activity_threshold := 0.15
@export var smooth_speed := 5.0

const MIN_PARTICLES := 8
const MAX_PARTICLES := 280

@onready var _mesh: MeshInstance3D = $Mesh
@onready var _particles: GPUParticles3D = $Particles

var activity_intensity := 0.35
var _target_intensity := 0.35
var _display_intensity := 0.35
var _material: ShaderMaterial


func _ready() -> void:
	_material = _mesh.material_override as ShaderMaterial
	_apply_shader_activity(_display_intensity)


func _process(delta: float) -> void:
	_display_intensity = lerpf(_display_intensity, _target_intensity, delta * smooth_speed)
	activity_intensity = _display_intensity

	rotate_y(rotation_speed * delta * (0.4 + _display_intensity * 0.8))
	if _material:
		_material.set_shader_parameter("time", Time.get_ticks_msec() / 1000.0)
		_apply_shader_activity(_display_intensity)
	_update_particles()


func update_brain_activity(intensity: float) -> void:
	_target_intensity = ActivityMapper.clamp_intensity(intensity)


func _apply_shader_activity(value: float) -> void:
	if _material:
		_material.set_shader_parameter("activity", value)


func _update_particles() -> void:
	if not _particles:
		return

	var above_threshold := _display_intensity >= particle_activity_threshold
	_particles.emitting = above_threshold
	if not above_threshold:
		_particles.amount = 0
		_particles.speed_scale = 0.0
		return

	var t := inverse_lerp(particle_activity_threshold, 1.0, _display_intensity)
	var amount := int(lerpf(float(MIN_PARTICLES), float(MAX_PARTICLES), t))
	_particles.amount = amount

	var process_mat: ParticleProcessMaterial = _particles.process_material
	if process_mat:
		process_mat.emission_sphere_radius = lerpf(1.02, 1.45, t)
		process_mat.initial_velocity_min = lerpf(0.15, 1.4, t)
		process_mat.initial_velocity_max = lerpf(0.6, 3.0, t)
		process_mat.scale_min = lerpf(0.015, 0.04, t)
		process_mat.scale_max = lerpf(0.04, 0.09, t)
	_particles.speed_scale = lerpf(0.5, 2.2, t)