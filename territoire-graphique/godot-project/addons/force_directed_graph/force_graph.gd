@tool
class_name ForceGraph
extends Control

signal node_clicked(memory_id: String)

@export var repulsion: float = 4200.0
@export var attraction: float = 0.012
@export var damping: float = 0.88
@export var node_radius: float = 10.0
@export var link_color: Color = Color(0.35, 0.55, 0.75, 0.45)
@export var node_color: Color = Color(0.55, 0.82, 1.0, 0.95)
@export var selected_color: Color = Color(1.0, 0.78, 0.35, 1.0)

var _hubs: Array = []
var _nodes: Array = []
var _selected_id: String = ""
var _sim_steps: int = 0

func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP
	resized.connect(_on_resized)
	_on_resized()

func _on_resized() -> void:
	queue_redraw()

func set_hubs(hubs: Array) -> void:
	_hubs = hubs.duplicate()
	_build_nodes()
	_sim_steps = 80
	queue_redraw()

func highlight_memory(memory_id: String) -> void:
	_selected_id = memory_id
	queue_redraw()

func _build_nodes() -> void:
	_nodes.clear()
	var center := size * 0.5
	var count := maxi(_hubs.size(), 1)
	var radius := minf(size.x, size.y) * 0.32
	for i in range(_hubs.size()):
		var hub: Dictionary = _hubs[i]
		var angle := TAU * float(i) / float(count)
		var pos := center + Vector2(cos(angle), sin(angle)) * radius
		_nodes.append({
			"id": str(hub.get("memory_id", "")),
			"title": str(hub.get("title", "")),
			"links": int(hub.get("inbound_links", 0)),
			"pos": pos,
			"vel": Vector2.ZERO,
		})

func _process(delta: float) -> void:
	if _sim_steps <= 0 or _nodes.is_empty():
		return
	var steps := mini(_sim_steps, 4)
	_sim_steps -= steps
	for _s in range(steps):
		_simulate_step()
	queue_redraw()

func _simulate_step() -> void:
	var center := size * 0.5
	for i in range(_nodes.size()):
		var n: Dictionary = _nodes[i]
		var force := Vector2.ZERO
		for j in range(_nodes.size()):
			if i == j:
				continue
			var other: Dictionary = _nodes[j]
			var delta_pos: Vector2 = n["pos"] - other["pos"]
			var dist_sq := maxf(delta_pos.length_squared(), 64.0)
			force += delta_pos.normalized() * (repulsion / dist_sq)
		force += (center - n["pos"]) * attraction
		n["vel"] = (n["vel"] + force) * damping
		n["pos"] += n["vel"]
		_nodes[i] = n

func _draw() -> void:
	if _nodes.is_empty():
		draw_string(
			ThemeDB.fallback_font,
			Vector2(12, 24),
			"Graphe vide — connectez le daemon.",
			HORIZONTAL_ALIGNMENT_LEFT,
			-1,
			12,
			Color(0.65, 0.72, 0.8)
		)
		return

	var center := size * 0.5
	for n in _nodes:
		var pos: Vector2 = n["pos"]
		draw_line(center, pos, link_color, 1.0)

	for n in _nodes:
		var pos: Vector2 = n["pos"]
		var col := selected_color if str(n["id"]) == _selected_id else node_color
		var r := node_radius + (3.0 if str(n["id"]) == _selected_id else 0.0)
		draw_circle(pos, r + 2.0, col.darkened(0.35))
		draw_circle(pos, r, col)
		var label := str(n["title"])
		if label.length() > 18:
			label = label.substr(0, 16) + "…"
		draw_string(ThemeDB.fallback_font, pos + Vector2(-36, -16), label, HORIZONTAL_ALIGNMENT_LEFT, 72, 10, Color(0.85, 0.9, 0.95))

func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var hit := _pick_node(event.position)
		if hit != "":
			_selected_id = hit
			node_clicked.emit(hit)
			queue_redraw()

func _pick_node(at: Vector2) -> String:
	var best_id := ""
	var best_dist := node_radius * 2.5
	for n in _nodes:
		var dist := at.distance_to(n["pos"])
		if dist < best_dist:
			best_dist = dist
			best_id = str(n["id"])
	return best_id