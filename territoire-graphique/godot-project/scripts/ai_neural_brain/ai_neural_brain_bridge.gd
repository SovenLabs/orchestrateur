extends Node
## Pont VisualEventMapper → AINeuralBrainSphere (optionnel dans MainTerritory).

@export var brain_path: NodePath
@export var enabled := true

var _brain: AINeuralBrainSphere


func _ready() -> void:
	_brain = get_node_or_null(brain_path) as AINeuralBrainSphere
	if not _brain:
		return
	var mapper := get_node_or_null("/root/VisualEventMapper")
	if mapper:
		mapper.effect_ready.connect(_on_visual_effect)
		mapper.activity_level_changed.connect(_on_activity_level)


func _on_activity_level(level: float) -> void:
	if enabled and _brain:
		_brain.set_agent_activity(clampf(level / 3.0, 0.0, 1.0))


func _on_visual_effect(effect: Dictionary) -> void:
	if not enabled or not _brain:
		return
	var kind := str(effect.get("kind", ""))
	match kind:
		"assimilation":
			_brain.set_agent_activity(0.9)
			_brain.stimulate_random_burst(40, 1.0, 0.85)
		"tool_call":
			_brain.set_agent_activity(0.75)
			_brain.stimulate_random_burst(18, 0.85, 0.5)
		"search", "graph_update":
			_brain.trigger_thought_propagation(_random_path(8))
		"thought_propagation":
			_brain.trigger_thought_propagation(_path_from_payload(effect))
		"neuron_stimulated":
			_stimulate_from_payload(effect)
		"error":
			_brain.set_agent_activity(0.35)
			_brain.stimulate_random_burst(60, 1.2, 0.35)
		"degraded":
			_brain.set_degraded_mode(true)
		"recovery", "pulse", "chat":
			_brain.set_agent_activity(0.55)


func _path_from_payload(effect: Dictionary) -> PackedInt32Array:
	var raw: Array = effect.get("path", [])
	var path := PackedInt32Array()
	for v in raw:
		path.append(int(v))
	return path if path.size() >= 2 else _random_path(8)


func _stimulate_from_payload(effect: Dictionary) -> void:
	var idx := int(effect.get("neuron_id", effect.get("id", -1)))
	var intensity := float(effect.get("intensity", 0.8))
	if idx >= 0:
		_brain.stimulate_neurons(PackedInt32Array([idx]), intensity, 0.6)
	else:
		_brain.stimulate_random_burst(12, intensity, 0.5)


func _random_path(length: int) -> PackedInt32Array:
	if _brain == null or _brain.neuron_count <= 0:
		return PackedInt32Array()
	var rng := RandomNumberGenerator.new()
	rng.randomize()
	var path := PackedInt32Array()
	var cursor := rng.randi_range(0, _brain.neuron_count - 1)
	path.append(cursor)
	for _i in range(length - 1):
		cursor = (cursor + rng.randi_range(3, 40)) % _brain.neuron_count
		path.append(cursor)
	return path