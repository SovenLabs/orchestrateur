# MainScene.gd — Fond spatial + UI Sci-Fi + bootstrap Territoire Graphique
# Godot 4.7.stable — doc: res://docs/GODOT_STABLE_REFERENCE.md
extends Node3D
class_name MainScene

const DEFAULT_WINDOW := Vector2i(1920, 1080)
const MIN_WINDOW_SIZE := Vector2i(1024, 600)
const STARFIELD_SHADER := preload("res://shaders/starfield_particles.gdshader")

@onready var world_env: WorldEnvironment = $WorldEnvironment
@onready var starfield: GPUParticles3D = $Background/Starfield
@onready var _camera: Camera3D = $Camera3D
@onready var _daemon: TerritoryDaemonClient = $DaemonClient

@onready var memory_panel: PanelContainer = %MemoryListPanel
@onready var chat_panel: PanelContainer = %ChatPanel
@onready var graph_panel: PanelContainer = %GraphPanel
@onready var monitoring_panel: PanelContainer = %MonitoringPanel

var _base_glow_intensity := 1.4
var _base_glow_bloom := 0.75


func _ready() -> void:
	_configure_window()
	WindowManager.register_main(self)
	setup_space_environment()
	setup_starfield()
	call_deferred("style_all_panels")

	if _daemon:
		_daemon.configure_window("main", ["chat", "memory", "graph", "monitoring"])

	var vp := get_viewport()
	if vp and not vp.size_changed.is_connected(_on_viewport_resized):
		vp.size_changed.connect(_on_viewport_resized)
	call_deferred("_on_viewport_resized")


func _process(delta: float) -> void:
	if starfield and starfield.material_override is ShaderMaterial:
		var mat := starfield.material_override as ShaderMaterial
		mat.set_shader_parameter("time", Time.get_ticks_msec() / 1000.0)


func set_activity(intensity: float, stress: float = 0.0) -> void:
	## Compat brain_sphere / territory_environment group.
	if world_env == null or world_env.environment == null:
		return
	var env := world_env.environment
	var t := clampf(intensity, 0.0, 1.0)
	env.glow_intensity = lerpf(_base_glow_intensity, _base_glow_intensity + 0.35, t)
	env.glow_bloom = lerpf(_base_glow_bloom, _base_glow_bloom + 0.1, t)
	if stress > 0.4:
		env.glow_intensity *= lerpf(1.0, 0.75, stress)


func setup_space_environment() -> void:
	var env := Environment.new()

	env.background_mode = Environment.BG_COLOR
	env.background_color = Color("#05050f")

	env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	env.ambient_light_color = Color(0.04, 0.06, 0.12)
	env.ambient_light_energy = 0.3

	env.glow_enabled = true
	env.glow_intensity = _base_glow_intensity
	env.glow_strength = 1.1
	env.glow_bloom = _base_glow_bloom
	env.glow_blend_mode = Environment.GLOW_BLEND_MODE_SOFTLIGHT
	env.glow_hdr_threshold = 0.6
	env.glow_hdr_scale = 1.8
	env.set("glow_levels/1", true)
	env.set("glow_levels/2", true)
	env.set("glow_levels/3", true)
	env.set("glow_levels/4", true)

	env.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	env.tonemap_exposure = 1.1
	env.adjustment_enabled = true
	env.adjustment_saturation = 1.1

	world_env.environment = env
	world_env.add_to_group("territory_environment")


func setup_starfield() -> void:
	if starfield == null:
		push_warning("Starfield node not found")
		return

	starfield.amount = 1200
	starfield.lifetime = 9999.0
	starfield.preprocess = 1200.0
	starfield.emitting = true
	starfield.visibility_aabb = AABB(Vector3(-60, -60, -60), Vector3(120, 120, 120))

	var process_mat := ParticleProcessMaterial.new()
	process_mat.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_BOX
	process_mat.emission_box_extents = Vector3(28, 18, 28)
	process_mat.direction = Vector3.ZERO
	process_mat.spread = 180.0
	process_mat.gravity = Vector3.ZERO
	process_mat.initial_velocity_min = 0.0
	process_mat.initial_velocity_max = 0.0
	process_mat.scale_min = 0.02
	process_mat.scale_max = 0.05
	starfield.process_material = process_mat
	starfield.material_override = create_starfield_shader_material()


func style_all_panels() -> void:
	apply_space_panel_style(memory_panel)
	apply_space_panel_style(chat_panel)
	apply_space_panel_style(graph_panel)
	apply_space_panel_style(monitoring_panel)


static func apply_space_panel_style(panel: PanelContainer) -> void:
	if panel == null:
		return

	var style := StyleBoxFlat.new()
	style.bg_color = Color("#0a0f1f")
	style.bg_color.a = 0.88
	style.border_color = Color("#00ddff")
	style.border_width_left = 2
	style.border_width_top = 2
	style.border_width_right = 2
	style.border_width_bottom = 2
	style.corner_radius_top_left = 10
	style.corner_radius_top_right = 10
	style.corner_radius_bottom_left = 10
	style.corner_radius_bottom_right = 10
	style.shadow_color = Color(0, 0, 0, 0.45)
	style.shadow_size = 12
	style.shadow_offset = Vector2(0, 4)
	style.content_margin_left = 4
	style.content_margin_top = 4
	style.content_margin_right = 4
	style.content_margin_bottom = 4
	panel.add_theme_stylebox_override("panel", style)


func create_starfield_shader_material() -> ShaderMaterial:
	var mat := ShaderMaterial.new()
	mat.shader = STARFIELD_SHADER
	mat.set_shader_parameter("time", 0.0)
	return mat


func _configure_window() -> void:
	var win := get_window()
	if win == null:
		return
	win.min_size = MIN_WINDOW_SIZE
	if win.size.x < MIN_WINDOW_SIZE.x or win.size.y < MIN_WINDOW_SIZE.y:
		win.size = DEFAULT_WINDOW


func _on_viewport_resized() -> void:
	if _camera == null:
		return
	var size := get_viewport().get_visible_rect().size
	if size.y <= 0.0:
		return
	var aspect := size.x / size.y
	_camera.fov = clampf(lerpf(58.0, 48.0, inverse_lerp(1.2, 2.2, aspect)), 44.0, 62.0)