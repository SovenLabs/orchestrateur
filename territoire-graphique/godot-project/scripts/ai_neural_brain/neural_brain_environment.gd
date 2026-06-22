class_name NeuralBrainEnvironment
extends WorldEnvironment
## WorldEnvironment cinématique — glow/bloom pour AI Neural Brain Sphere.

@export_category("Glow HDR")
@export_range(0.0, 2.0, 0.01) var glow_intensity := 0.82
@export_range(0.0, 2.0, 0.01) var glow_strength := 1.15
@export_range(0.0, 1.0, 0.01) var glow_bloom := 0.38
@export_range(0.0, 2.0, 0.01) var glow_hdr_threshold := 0.55
@export_range(0.0, 4.0, 0.01) var glow_hdr_scale := 2.4


func _ready() -> void:
	_apply()


func apply_cinematic_glow() -> void:
	_apply()


func set_activity_boost(level: float) -> void:
	if not environment:
		return
	var t := clampf(level, 0.0, 1.0)
	environment.glow_intensity = lerpf(glow_intensity, glow_intensity + 0.35, t)
	environment.glow_bloom = lerpf(glow_bloom, glow_bloom + 0.12, t)


func _apply() -> void:
	if environment == null:
		environment = Environment.new()
	environment.background_mode = Environment.BG_SKY
	environment.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	environment.tonemap_exposure = 1.08
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	environment.ambient_light_color = Color(0.04, 0.06, 0.12)
	environment.ambient_light_energy = 0.35
	environment.glow_enabled = true
	environment.glow_intensity = glow_intensity
	environment.glow_strength = glow_strength
	environment.glow_bloom = glow_bloom
	environment.glow_blend_mode = Environment.GLOW_BLEND_MODE_SCREEN
	environment.glow_hdr_threshold = glow_hdr_threshold
	environment.glow_hdr_scale = glow_hdr_scale
	environment.set("glow_levels/1", true)
	environment.set("glow_levels/2", true)
	environment.set("glow_levels/3", true)
	environment.set("glow_levels/4", true)
	environment.set("glow_levels/5", false)
	environment.adjustment_enabled = true
	environment.adjustment_brightness = 1.0
	environment.adjustment_contrast = 1.08
	environment.adjustment_saturation = 1.18