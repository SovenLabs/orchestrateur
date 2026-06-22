extends Node3D
class_name BrainSphere
## Boule de Pixels Vivante — réactivité temps réel Phase 20.

enum ParticleMode { IDLE, ASSIMILATION, TOOL_CALL }

@export var rotation_speed := 0.35
@export var particle_activity_threshold := 0.12
@export var smooth_speed := 5.0

const MIN_PARTICLES := 6
const MAX_IDLE_PARTICLES := 120
const MAX_ASSIM_PARTICLES := 140
const MAX_TOOL_PARTICLES := 80
const MAX_TOTAL_PARTICLES := 300

@onready var _mesh: MeshInstance3D = $Mesh
@onready var _particles_idle: GPUParticles3D = $ParticlesIdle
@onready var _particles_assim: GPUParticles3D = $ParticlesAssimilation
@onready var _particles_tool: GPUParticles3D = $ParticlesTool

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
var _particle_mode := ParticleMode.IDLE
var _particle_mode_timer := 0.0
var _living := 1.0
var _target_living := 1.0
var _material: ShaderMaterial


func _ready() -> void:
	_material = _mesh.material_override as ShaderMaterial
	_apply_shader_state()
	var mapper := get_node_or_null("/root/VisualEventMapper")
	if mapper:
		mapper.visual_effect_triggered.connect(_on_visual_effect_signal)
		mapper.effect_ready.connect(apply_visual_effect)
		mapper.activity_level_changed.connect(_on_mapper_activity)


func _process(delta: float) -> void:
	if _pulse_remaining > 0.0:
		_pulse_remaining = maxf(0.0, _pulse_remaining - delta)
	if _swirl_timer > 0.0:
		_swirl_timer = maxf(0.0, _swirl_timer - delta)
	if _rotation_flash > 0.0:
		_rotation_flash = maxf(0.0, _rotation_flash - delta * 2.5)
	if _shake_timer > 0.0:
		_shake_timer = maxf(0.0, _shake_timer - delta)
	if _particle_mode_timer > 0.0:
		_particle_mode_timer = maxf(0.0, _particle_mode_timer - delta)
		if _particle_mode_timer <= 0.0 and _particle_mode != ParticleMode.IDLE:
			_particle_mode = ParticleMode.IDLE

	_stress = lerpf(_stress, _target_stress, delta * 4.0)
	_living = lerpf(_living, _target_living, delta * 2.5)

	var pulse := 0.0
	if _pulse_remaining > 0.0 and _pulse_duration > 0.0:
		pulse = _pulse_boost * (_pulse_remaining / _pulse_duration)

	var effective := minf(1.0, _target_intensity + pulse)
	if _degraded:
		effective *= 0.55
	_display_intensity = lerpf(_display_intensity, effective, delta * smooth_speed)
	activity_intensity = _display_intensity

	var rot_boost := 1.0 + _rotation_flash * 2.0
	if _swirl_timer > 0.0:
		rot_boost += 0.6
	var rot_speed := rotation_speed * (0.35 + _display_intensity * 0.85) * rot_boost
	if _degraded:
		rot_speed *= 0.45
	rotate_y(rot_speed * delta)

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
	_target_living = 0.35 if active else 1.0


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
	var mode_name := str(effect.get("particle_mode", ""))
	_set_particle_mode_from_name(mode_name, float(effect.get("intensity", 0.6)))
	var kind: String = str(effect.get("kind", ""))
	if kind == "degraded":
		set_degraded_mode(true)
	elif kind in ["assimilation", "tool_call", "search", "pulse", "chat", "recovery", "graph_update"]:
		set_degraded_mode(false)


func _on_visual_effect_signal(effect_name: String, intensity: float) -> void:
	match effect_name:
		"activity_change":
			update_brain_activity(intensity)
		"error":
			_target_stress = maxf(_target_stress, 0.85)
		"degraded":
			set_degraded_mode(true)
		"recovery":
			set_degraded_mode(false)


func _on_mapper_activity(level: float) -> void:
	update_brain_activity(level / 3.0)


func _set_particle_mode_from_name(mode_name: String, intensity: float) -> void:
	match mode_name:
		"assimilation":
			_particle_mode = ParticleMode.ASSIMILATION
			_particle_mode_timer = 1.2
		"tool_call":
			_particle_mode = ParticleMode.TOOL_CALL
			_particle_mode_timer = 0.55
			_burst_tool_particles(intensity)
		_:
			if _particle_mode_timer <= 0.0:
				_particle_mode = ParticleMode.IDLE


func _burst_tool_particles(intensity: float) -> void:
	if not _particles_tool:
		return
	_particles_tool.restart()
	_particles_tool.emitting = true
	_particles_tool.amount = int(lerpf(20.0, float(MAX_TOOL_PARTICLES), intensity))


func _apply_shader_state() -> void:
	if not _material:
		return
	_material.set_shader_parameter("time", Time.get_ticks_msec() / 1000.0)
	_material.set_shader_parameter("activity", _display_intensity)
	_material.set_shader_parameter("stress", _stress)
	_material.set_shader_parameter("living", _living)
	var swirl_val := 1.0 if _swirl_timer > 0.0 else 0.0
	_material.set_shader_parameter("swirl", swirl_val)
	var refract := 0.02 if _degraded else 0.04 + _display_intensity * 0.03
	_material.set_shader_parameter("refraction_strength", refract)


func _update_particles() -> void:
	var t := inverse_lerp(particle_activity_threshold, 1.0, _display_intensity)
	var budget := MAX_TOTAL_PARTICLES

	_update_idle_particles(t, budget)
	_update_assimilation_particles(t, budget)
	_update_tool_particles(t)


func _update_idle_particles(t: float, budget: int) -> void:
	if not _particles_idle:
		return
	var always_on := _display_intensity >= particle_activity_threshold or _degraded
	_particles_idle.emitting = always_on or _particle_mode == ParticleMode.IDLE
	if not _particles_idle.emitting:
		_particles_idle.amount = 0
		_particles_idle.speed_scale = 0.0
		return

	var amount := int(lerpf(float(MIN_PARTICLES), float(MAX_IDLE_PARTICLES), t))
	if _degraded:
		amount = int(amount * 0.4)
	if _pulse_remaining > 0.0:
		amount = int(amount * 1.15)
	_particles_idle.amount = mini(amount, budget)

	var process_mat: ParticleProcessMaterial = _particles_idle.process_material
	if process_mat:
		process_mat.emission_sphere_radius = lerpf(1.05, 1.4, t)
		process_mat.initial_velocity_min = lerpf(0.08, 0.5, t)
		process_mat.initial_velocity_max = lerpf(0.25, 1.2, t)
		process_mat.scale_min = lerpf(0.012, 0.03, t)
		process_mat.scale_max = lerpf(0.03, 0.07, t)
	var speed := lerpf(0.35, 1.4, t)
	if _degraded:
		speed *= 0.35
	_particles_idle.speed_scale = speed


func _update_assimilation_particles(t: float, budget: int) -> void:
	if not _particles_assim:
		return
	var active := _particle_mode == ParticleMode.ASSIMILATION or _swirl_timer > 0.0
	_particles_assim.emitting = active
	if not active:
		_particles_assim.amount = 0
		return

	var amount := int(lerpf(30.0, float(MAX_ASSIM_PARTICLES), t))
	if _pulse_remaining > 0.0:
		amount = int(amount * 1.3)
	_particles_assim.amount = mini(amount, budget)

	var process_mat: ParticleProcessMaterial = _particles_assim.process_material
	if process_mat:
		process_mat.emission_sphere_radius = lerpf(1.35, 1.75, t)
		process_mat.radial_accel_min = lerpf(-3.5, -6.0, t)
		process_mat.radial_accel_max = lerpf(-2.0, -4.5, t)
		process_mat.initial_velocity_min = 0.05
		process_mat.initial_velocity_max = lerpf(0.2, 0.8, t)
	_particles_assim.speed_scale = lerpf(1.0, 2.2, t)


func _update_tool_particles(t: float, _budget: int) -> void:
	if not _particles_tool:
		return
	if _particle_mode != ParticleMode.TOOL_CALL and _shake_timer <= 0.0:
		if _particles_tool.emitting and _particle_mode_timer <= 0.0:
			_particles_tool.emitting = false
		return

	var process_mat: ParticleProcessMaterial = _particles_tool.process_material
	if process_mat:
		process_mat.emission_sphere_radius = lerpf(0.9, 1.1, t)
		process_mat.initial_velocity_min = lerpf(1.5, 3.5, t)
		process_mat.initial_velocity_max = lerpf(3.0, 6.0, t)
	_particles_tool.speed_scale = lerpf(1.8, 3.5, t)