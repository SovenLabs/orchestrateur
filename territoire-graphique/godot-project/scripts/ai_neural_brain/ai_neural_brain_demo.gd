extends Node3D
## Démo autonome — AI Neural Brain Sphere + hooks de test clavier.

@onready var _brain: AINeuralBrainSphere = $AINeuralBrainSphere
@onready var _env: NeuralBrainEnvironment = $WorldEnvironment


func _ready() -> void:
	_setup_sky()
	if _env:
		_env.apply_cinematic_glow()


func _unhandled_input(event: InputEvent) -> void:
	if not _brain:
		return
	if event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_1:
				_brain.set_agent_activity(0.25)
			KEY_2:
				_brain.set_agent_activity(0.55)
			KEY_3:
				_brain.set_agent_activity(0.95)
			KEY_SPACE:
				_brain.stimulate_random_burst(32, 1.0, 0.75)
			KEY_T:
				var path := PackedInt32Array([0, 12, 48, 120, 300, 600])
				_brain.trigger_thought_propagation(path)


func _process(delta: float) -> void:
	if _brain and _env:
		_env.set_activity_boost(_brain.agent_activity)


func _setup_sky() -> void:
	var env_node := $WorldEnvironment as WorldEnvironment
	if env_node == null or env_node.environment == null:
		return
	var sky_shader := load("res://shaders/ai_neural_brain/starfield_background.gdshader") as Shader
	if sky_shader == null:
		return
	var sky_mat := ShaderMaterial.new()
	sky_mat.shader = sky_shader
	var sky := Sky.new()
	sky.sky_material = sky_mat
	env_node.environment.sky = sky
	env_node.environment.background_mode = Environment.BG_SKY