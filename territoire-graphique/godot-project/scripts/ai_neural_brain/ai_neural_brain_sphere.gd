# AI Neural Brain Sphere — composant visuel premium sci-fi
#
# === Connexion agent LLM externe (exemple) ===
#   var brain := $AINeuralBrainSphere
#   brain.set_agent_activity(agent_load)           # 0..1 charge agent
#   brain.stimulate_neurons(indices, 1.0, 0.6)     # après tool_call
#   brain.trigger_thought_propagation(path_ids)    # chaîne de neurones activés
#
#   # Depuis VisualEventMapper / WebSocket :
#   mapper.effect_ready.connect(func(e):
#       if e.get("kind") == "assimilation":
#           brain.set_agent_activity(0.85)
#           brain.stimulate_random_burst(24, 1.0, 0.8)
#   )
#
@tool
class_name AINeuralBrainSphere
extends Node3D

const CORE_SHADER := preload("res://shaders/ai_neural_brain/core_plasma.gdshader")
const NEURON_SHADER := preload("res://shaders/ai_neural_brain/neuron_multimesh.gdshader")
const SYNAPSE_SHADER := preload("res://shaders/ai_neural_brain/synapse_multimesh.gdshader")

@export_category("Génération")
@export_range(400, 2500, 1) var neuron_count := 1400
@export_range(3, 8, 1) var neighbors_k := 5
@export var sphere_seed: int = 1337
@export var shell_radius: float = 3.7
@export var max_edge_length: float = 0.0
@export var auto_generate_on_ready := true
@export var editor_preview := true

@export_category("Noyau AI")
@export var core_radius: float = 0.85
@export var core_color_a: Color = Color("#ccffff")
@export var core_color_b: Color = Color("#aaddff")

@export_category("Neurones")
@export var neuron_base_size: float = 0.04
@export var warm_accent_ratio: float = 0.2
@export var magenta_accent_ratio: float = 0.05

@export_category("Synapses")
@export var connection_width: float = 0.018
@export var flow_speed: float = 2.4

@export_category("Activité")
@export_range(0.0, 1.0, 0.01) var agent_activity: float = 0.45
@export var auto_rotate_core := true
@export var core_rotate_speed: float = 0.18

@export_category("Éditeur")
@export var regenerate_now: bool = false:
	set(value):
		regenerate_now = value
		if value and Engine.is_editor_hint():
			generate(sphere_seed)
			regenerate_now = false

@onready var _core_mesh: MeshInstance3D = $Core
@onready var _neurons_mm: MultiMeshInstance3D = $Neurons
@onready var _synapses_mm: MultiMeshInstance3D = $Synapses

var _neuron_positions: PackedVector3Array = PackedVector3Array()
var _edges: PackedVector2iArray = PackedVector2iArray()
var _edge_lookup: Dictionary = {}
var _edge_spikes: PackedFloat32Array = PackedFloat32Array()

var _core_material: ShaderMaterial
var _neuron_material: ShaderMaterial
var _synapse_material: ShaderMaterial

var _neuron_spikes: PackedFloat32Array = PackedFloat32Array()
var _neuron_spike_decay: PackedFloat32Array = PackedFloat32Array()
var _propagations: Array = []
var _network_activity := 1.0
var _activity_shader := 1.0
var _generated := false


func _ready() -> void:
	_setup_materials()
	if auto_generate_on_ready and (not Engine.is_editor_hint() or editor_preview):
		if not _generated:
			generate(sphere_seed)


func _process(delta: float) -> void:
	var t := Time.get_ticks_msec() / 1000.0
	_activity_shader = lerpf(0.3, 2.8, clampf(agent_activity, 0.0, 1.0))
	_network_activity = lerpf(0.65, 1.85, clampf(agent_activity, 0.0, 1.0))

	if _core_material:
		_core_material.set_shader_parameter("time", t)
		_core_material.set_shader_parameter("activity", _activity_shader)
	if _neuron_material:
		_neuron_material.set_shader_parameter("time", t)
		_neuron_material.set_shader_parameter("network_activity", _network_activity)
	if _synapse_material:
		_synapse_material.set_shader_parameter("time", t)
		_synapse_material.set_shader_parameter("network_activity", _network_activity)
		_synapse_material.set_shader_parameter("flow_speed", flow_speed)

	if auto_rotate_core:
		_core_mesh.rotate_y(core_rotate_speed * delta)

	_tick_neuron_spikes(delta)
	_tick_propagations(delta)

	if Engine.is_editor_hint():
		return


# --- API publique (hooks IA) ---

func set_agent_activity(level: float) -> void:
	agent_activity = clampf(level, 0.0, 1.0)


func stimulate_neurons(indices: PackedInt32Array, intensity: float, duration: float) -> void:
	if indices.is_empty():
		return
	var boost := clampf(intensity, 0.0, 3.0)
	var decay := 1.0 / maxf(0.05, duration)
	for idx in indices:
		if idx < 0 or idx >= _neuron_spikes.size():
			continue
		_neuron_spikes[idx] = maxf(_neuron_spikes[idx], boost)
		_neuron_spike_decay[idx] = decay
	_stimulate_edges_for_neurons(indices, boost * 0.65)


func trigger_thought_propagation(path: PackedInt32Array) -> void:
	if path.size() < 2:
		return
	_propagations.append({
		"path": path.duplicate(),
		"progress": 0.0,
		"speed": 2.2,
	})


func stimulate_random_burst(count: int, intensity: float, duration: float) -> void:
	if _neuron_positions.is_empty():
		return
	var rng := RandomNumberGenerator.new()
	rng.seed = int(Time.get_ticks_msec())
	var picked := PackedInt32Array()
	for _i in range(mini(count, _neuron_positions.size())):
		picked.append(rng.randi_range(0, _neuron_positions.size() - 1))
	stimulate_neurons(picked, intensity, duration)


func generate(sphere_seed_value: int) -> void:
	sphere_seed = sphere_seed_value
	var rng := RandomNumberGenerator.new()
	rng.seed = sphere_seed

	_neuron_positions = NeuralGraphGenerator.fibonacci_sphere(neuron_count, shell_radius)
	_edges = NeuralGraphGenerator.build_knn_edges(
		_neuron_positions,
		neighbors_k,
		max_edge_length,
		sphere_seed,
	)
	_build_edge_lookup()
	_build_neuron_multimesh(rng)
	_build_synapse_multimesh()
	_generated = true

	if not Engine.is_editor_hint():
		print(
			"AINeuralBrainSphere: %d neurones, %d synapses (seed=%d)"
			% [_neuron_positions.size(), _edges.size(), sphere_seed]
		)


# --- Internals ---

func _setup_materials() -> void:
	_core_material = ShaderMaterial.new()
	_core_material.shader = CORE_SHADER
	_core_material.set_shader_parameter("core_color_a", Vector3(core_color_a.r, core_color_a.g, core_color_a.b))
	_core_material.set_shader_parameter("core_color_b", Vector3(core_color_b.r, core_color_b.g, core_color_b.b))
	_core_mesh.material_override = _core_material

	_neuron_material = ShaderMaterial.new()
	_neuron_material.shader = NEURON_SHADER
	_neurons_mm.material_override = _neuron_material

	_synapse_material = ShaderMaterial.new()
	_synapse_material.shader = SYNAPSE_SHADER
	_synapses_mm.material_override = _synapse_material

	var core_sphere := SphereMesh.new()
	core_sphere.radius = core_radius
	core_sphere.height = core_radius * 2.0
	core_sphere.radial_segments = 48
	core_sphere.rings = 32
	_core_mesh.mesh = core_sphere


func _build_neuron_multimesh(rng: RandomNumberGenerator) -> void:
	var mm := MultiMesh.new()
	var neuron_mesh := SphereMesh.new()
	neuron_mesh.radius = neuron_base_size
	neuron_mesh.height = neuron_base_size * 2.0
	neuron_mesh.radial_segments = 6
	neuron_mesh.rings = 4

	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.use_colors = true
	mm.use_custom_data = true
	mm.mesh = neuron_mesh
	mm.instance_count = _neuron_positions.size()

	_neuron_spikes.resize(_neuron_positions.size())
	_neuron_spike_decay.resize(_neuron_positions.size())
	_neuron_spikes.fill(0.0)
	_neuron_spike_decay.fill(0.0)

	for i in range(_neuron_positions.size()):
		var pos := _neuron_positions[i]
		var size_scale := rng.randf_range(0.85, 1.15)
		var phase := NeuralGraphGenerator.hash01(i, sphere_seed, 17)
		var accent := 0.0
		var roll := rng.randf()
		if roll < magenta_accent_ratio:
			accent = rng.randf_range(0.66, 1.0)
		elif roll < magenta_accent_ratio + warm_accent_ratio:
			accent = rng.randf_range(0.33, 0.66)

		var basis := Basis.IDENTITY.scaled(Vector3.ONE * size_scale)
		mm.set_instance_transform(i, Transform3D(basis, pos))

		var col := Color(0.0, 0.87, 1.0, 1.0)
		if accent > 0.66:
			col = Color(1.0, 0.73, 0.27, 1.0)
		elif accent > 0.33:
			col = Color(0.73, 0.53, 1.0, 1.0)
		mm.set_instance_color(i, col)
		mm.set_instance_custom_data(i, Color(phase, size_scale, accent, 0.0))

	_neurons_mm.multimesh = mm


func _build_synapse_multimesh() -> void:
	var mm := MultiMesh.new()
	var ribbon := BoxMesh.new()
	ribbon.size = Vector3(1.0, 1.0, 1.0)

	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.use_colors = false
	mm.use_custom_data = true
	mm.mesh = ribbon
	mm.instance_count = _edges.size()

	_edge_spikes.resize(_edges.size())
	_edge_spikes.fill(0.0)

	for edge_i in range(_edges.size()):
		var e := _edges[edge_i]
		var a := _neuron_positions[e.x]
		var b := _neuron_positions[e.y]
		mm.set_instance_transform(edge_i, _ribbon_transform(a, b))
		mm.set_instance_custom_data(edge_i, Color(0.0, 0.0, 0.0, 0.0))

	_synapses_mm.multimesh = mm


func _ribbon_transform(a: Vector3, b: Vector3) -> Transform3D:
	var dir := b - a
	var length := dir.length()
	if length < 0.0001:
		return Transform3D(Basis.IDENTITY, a)
	dir /= length
	var up := Vector3.UP
	if absf(dir.dot(up)) > 0.92:
		up = Vector3.RIGHT
	var side := dir.cross(up).normalized()
	var forward := dir
	var basis := Basis(side * connection_width, up * connection_width, forward * length)
	return Transform3D(basis, (a + b) * 0.5)


func _build_edge_lookup() -> void:
	_edge_lookup.clear()
	for i in range(_edges.size()):
		var e := _edges[i]
		var key := "%d:%d" % [e.x, e.y]
		_edge_lookup[key] = i


func _edge_index_for(a: int, b: int) -> int:
	var key := "%d:%d" % [mini(a, b), maxi(a, b)]
	return int(_edge_lookup.get(key, -1))


func _stimulate_edges_for_neurons(indices: PackedInt32Array, amount: float) -> void:
	var dirty := false
	for idx in indices:
		for edge_i in range(_edges.size()):
			var e := _edges[edge_i]
			if e.x != idx and e.y != idx:
				continue
			_edge_spikes[edge_i] = maxf(_edge_spikes[edge_i], amount)
			dirty = true
	if dirty:
		_push_edge_spikes_to_gpu()


func _tick_neuron_spikes(delta: float) -> void:
	if _neurons_mm.multimesh == null:
		return
	var dirty := false
	for i in range(_neuron_spikes.size()):
		if _neuron_spikes[i] <= 0.0:
			continue
		_neuron_spikes[i] = maxf(0.0, _neuron_spikes[i] - _neuron_spike_decay[i] * delta)
		var custom := _neurons_mm.multimesh.get_instance_custom_data(i)
		custom.a = _neuron_spikes[i]
		_neurons_mm.multimesh.set_instance_custom_data(i, custom)
		dirty = true
	if dirty and _has_active_neuron_spikes():
		_decay_edge_spikes(delta)


func _decay_edge_spikes(delta: float) -> void:
	var dirty := false
	for i in range(_edge_spikes.size()):
		if _edge_spikes[i] <= 0.0:
			continue
		_edge_spikes[i] = maxf(0.0, _edge_spikes[i] - delta * 1.6)
		dirty = true
	if dirty:
		_push_edge_spikes_to_gpu()


func _tick_propagations(delta: float) -> void:
	if _propagations.is_empty():
		return
	var finished: Array = []
	for p in _propagations:
		p["progress"] = float(p["progress"]) + float(p["speed"]) * delta
		var path: PackedInt32Array = p["path"]
		var wave := float(p["progress"])
		for seg in range(path.size() - 1):
			var edge_i := _edge_index_for(path[seg], path[seg + 1])
			if edge_i < 0:
				continue
			var dist := absf(float(seg) - wave)
			if dist < 0.45:
				_edge_spikes[edge_i] = maxf(_edge_spikes[edge_i], 1.0 - dist)
		if wave > float(path.size()):
			finished.append(p)
	for done in finished:
		_propagations.erase(done)
	_push_edge_spikes_to_gpu()


func _push_edge_spikes_to_gpu() -> void:
	var mm := _synapses_mm.multimesh
	if mm == null:
		return
	for edge_i in range(_edge_spikes.size()):
		var custom := mm.get_instance_custom_data(edge_i)
		custom.a = _edge_spikes[edge_i]
		mm.set_instance_custom_data(edge_i, custom)


func _has_active_neuron_spikes() -> bool:
	for v in _neuron_spikes:
		if v > 0.01:
			return true
	return false