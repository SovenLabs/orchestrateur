extends WorldEnvironment
## Pilote le post-processing (glow HDR) selon l'activité de la boule.

@export var base_glow_intensity := 0.55
@export var max_glow_intensity := 1.35
@export var base_bloom := 0.08
@export var max_bloom := 0.22

var _target_intensity := 0.55
var _target_bloom := 0.08


func _ready() -> void:
	add_to_group("territory_environment")
	if not environment:
		environment = Environment.new()
		_configure_environment(environment)
	_apply_glow(_target_intensity, _target_bloom)


func set_activity(intensity: float, stress: float = 0.0) -> void:
	var t := clampf(intensity, 0.0, 1.0)
	_target_intensity = lerpf(base_glow_intensity, max_glow_intensity, t)
	_target_bloom = lerpf(base_bloom, max_bloom, t)
	if stress > 0.4:
		_target_intensity = lerpf(_target_intensity, _target_intensity * 0.7, stress)
		_target_bloom = lerpf(_target_bloom, _target_bloom * 1.25, stress * 0.5)


func _process(delta: float) -> void:
	if not environment:
		return
	var current := environment.glow_intensity
	var current_bloom := environment.glow_bloom
	environment.glow_intensity = lerpf(current, _target_intensity, delta * 4.0)
	environment.glow_bloom = lerpf(current_bloom, _target_bloom, delta * 4.0)


func _configure_environment(env: Environment) -> void:
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color(0.02, 0.03, 0.08)
	env.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	env.tonemap_exposure = 1.05
	env.glow_enabled = true
	env.glow_intensity = base_glow_intensity
	env.glow_strength = 1.05
	env.glow_bloom = base_bloom
	env.glow_blend_mode = Environment.GLOW_BLEND_MODE_SOFTLIGHT
	env.glow_hdr_threshold = 0.85
	env.glow_hdr_scale = 2.0
	env.glow_levels/1 = true
	env.glow_levels/2 = true
	env.glow_levels/3 = true
	env.glow_levels/4 = false
	env.adjustment_enabled = true
	env.adjustment_brightness = 1.0
	env.adjustment_contrast = 1.05
	env.adjustment_saturation = 1.12