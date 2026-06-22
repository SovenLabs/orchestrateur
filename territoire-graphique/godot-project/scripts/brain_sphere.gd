extends Node3D
class_name BrainSphere
## Boule de Pixels Vivante — réactivité temps réel Phase 19.

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
var _pulse_boost := 0.0
var _pulse_remaining := 0.0
var _pulse_duration := 0.0
var _stress := 0.0
var _target_stress := 0.0
var _swirl_timer := 0.0
var _rotation_flash := 0.0
var _shake_timer := 0.0
var _degraded := false
var _material: ShaderMaterial


func _ready() -> void:
	_material = _mesh.material_override as ShaderMaterial
	_apply_shader_state()


func _process(delta: float) -> void:
	if _pulse_remaining > 0.0:
		_pulse_remaining = maxf(0.0, _pulse_remaining - delta)
	if _swirl_timer > 0.0:
		_swirl_timer = maxf(0.0, _swirl_timer - delta)
	if _rotation_flash > 0.0:
		_rotation_flash = maxf(0.0, _rotation_flash - delta * 2.5)
	if _shake_timer > 0.0:
		_shake_timer = maxf(0.0, _shake_timer - delta)

	_stress = lerpf(_stress, _target_stress, delta * 4.0)

	var pulse := 0.0
	if _pulse_remaining > 0.0 and _pulse_duration > 0.0:
		pulse = _pulse_boost * (_pulse_remaining / _pulse_duration)

	var effective := minf(1.0, _target_intensity + pulse)
	if _degraded:
		effective *= 0.65
	_display_intensity = lerpf(_display_intensity, effective, delta * smooth_speed)
	activity_intensity = _display_intensity

	var rot_boost := 1.0 + _rotation_flash * 2.0
	if _swirl_timer > 0.0:
		rot_boost += 0.6
	rotate_y(rotation_speed * delta * (0.4 + _display_intensity * 0.8) * rot_boost)

	if _shake_timer > 0.0:
		_mesh.position = Vector3(
			sin(Time.get_ticks_msec() * 0.04) * 0.03 * _stress,
			cos(Time.get_ticks_msec() * 0.035) * 0.02 * _stress,
			0.0,
		)
	else:
		_mesh.position = Vector3.ZERO

	_apply_shader_state()
	_update_particles()


func update_brain_activity(intensity: float) -> void:
	_target_intensity = ActivityMapper.clamp_intensity(intensity)


func set_degraded_mode(active: bool) -> void:
	_degraded = active
	_target_stress = 0.55 if active else 0.0


func pulse_activity(boost: float, duration: float) -> void:
	_pulse_boost = boost
	_pulse_duration = duration
	_pulse_remaining = maxf(_pulse_remaining, duration)


func apply_visual_effect(effect: Dictionary) -> void:
	if effect.is_empty():
		return
	if effect.has("pulse_boost"):
		pulse_activity(
			float(effect.get("pulse_boost", 0.4)),
			float(effect.get("pulse_duration", 0.5)),
		)
	if bool(effect.get("swirl", false)):
		_swirl_timer = 0.6
	if effect.has("stress"):
		_target_stress = float(effect.get("stress", 0.0))
	if float(effect.get("flash_rotation", 0.0)) > 0.0:
		_rotation_flash = float(effect.get("flash_rotation", 0.0))
	if bool(effect.get("shake", false)):
		_shake_timer = 0.45
	var kind: String = str(effect.get("kind", ""))
	if kind == "degraded":
		set_degraded_mode(true)
	elif kind in ["assimilation", "tool_call", "search", "pulse", "chat"]:
		set_degraded_mode(false)


func _apply_shader_state() -> void:
	if not _material:
		return
	_material.set_shader_parameter("time", Time.get_ticks_msec() / 1000.0)
	_material.set_shader_parameter("activity", _display_intensity)
	_material.set_shader_parameter("stress", _stress)
	var swirl := 1.0 if _swirl_timer > 0.0 else 0.0
	_material.set_shader_parameter("swirl", swirl)


func _update_particles() -> void:
	if not _particles:
		return

	var above_threshold := (
		_display_intensity >= particle_activity_threshold
		or _pulse_remaining > 0.0
		or _swirl_timer > 0.0
	)
	_particles.emitting = above_threshold
	if not above_threshold:
		_particles.amount = 0
		_particles.speed_scale = 0.0
		return

	var t := inverse_lerp(particle_activity_threshold, 1.0, _display_intensity)
	var amount := int(lerpf(float(MIN_PARTICLES), float(MAX_PARTICLES), t))
	if _pulse_remaining > 0.0:
		amount = int(amount * 1.25)
	if _swirl_timer > 0.0:
		amount = int(amount * 1.35)
	_particles.amount = mini(amount, MAX_PARTICLES)

	var process_mat: ParticleProcessMaterial = _particles.process_material
	if process_mat:
		var radius := lerpf(1.02, 1.45, t)
		if _swirl_timer > 0.0:
			radius *= 1.15
		process_mat.emission_sphere_radius = radius
		process_mat.initial_velocity_min = lerpf(0.15, 1.4, t)
		process_mat.initial_velocity_max = lerpf(0.6, 3.0, t)
		process_mat.scale_min = lerpf(0.015, 0.04, t)
		process_mat.scale_max = lerpf(0.04, 0.09, t)
	var speed := lerpf(0.5, 2.2, t)
	if _swirl_timer > 0.0:
		speed *= 1.4
	_particles.speed_scale = speed