extends Node
class_name AgentInteraction
## Sélection et actions sur les sphères agents (raycast + commandes bridge).

signal agent_selected(agent_id: String)

@export var camera: Camera3D
@export var territory_agents: TerritoryAgents

var _daemon: TerritoryDaemonClient
var _selected_id := ""


func _ready() -> void:
	_daemon = TerritoryDaemonClient.resolve(self)
	if territory_agents:
		territory_agents.agent_clicked.connect(_on_agent_clicked)
	if camera == null:
		camera = get_parent().get_node_or_null("Camera3D") as Camera3D


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		_try_pick(event.position)


func _try_pick(screen_pos: Vector2) -> void:
	if camera == null or territory_agents == null:
		return
	var from := camera.project_ray_origin(screen_pos)
	var dir := camera.project_ray_normal(screen_pos)
	var space := camera.get_world_3d().direct_space_state
	var query := PhysicsRayQueryParameters3D.create(from, from + dir * 80.0)
	var hit := space.intersect_ray(query)
	if hit.is_empty():
		return
	var collider: Node = hit.get("collider")
	if collider == null:
		return
	var parent := collider.get_parent()
	if parent is AgentSphere:
		_on_agent_clicked((parent as AgentSphere).agent_id)


func _on_agent_clicked(agent_id: String) -> void:
	_selected_id = agent_id
	agent_selected.emit(agent_id)
	if _daemon:
		_daemon.execute_get_agent(agent_id)


func wake_selected() -> void:
	if _selected_id.is_empty() or _daemon == null:
		return
	_daemon.execute_agent_wake(_selected_id)


func sleep_selected() -> void:
	if _selected_id.is_empty() or _daemon == null:
		return
	_daemon.execute_agent_sleep(_selected_id)