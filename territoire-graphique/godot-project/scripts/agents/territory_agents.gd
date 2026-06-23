extends Node3D
class_name TerritoryAgents
## Conteneur orbital des AgentSphere — bootstrap ListAgents + événements temps réel.

signal agent_clicked(agent_id: String)

const AGENT_SPHERE_SCRIPT := preload("res://scripts/agents/agent_sphere.gd")
const ORBIT_RADIUS := 4.5

@export var communication_lines: CommunicationLines

var _daemon: TerritoryDaemonClient
var _spheres: Dictionary = {}


func _ready() -> void:
	_daemon = TerritoryDaemonClient.resolve(self)
	if _daemon:
		_daemon.command_completed.connect(_on_command_completed)
		_daemon.broadcast_received.connect(_on_broadcast)
		_daemon.connection_state_changed.connect(_on_connection_state_changed)
		if _daemon.is_ready():
			_request_agent_list()


func _on_connection_state_changed(connected: bool, _detail: String) -> void:
	if connected:
		_request_agent_list()


func _request_agent_list() -> void:
	if _daemon and _daemon.is_ready():
		_daemon.execute_list_agents()


func _on_command_completed(_rid: String, response: Dictionary) -> void:
	var kind: String = str(response.get("response", ""))
	if kind != "AgentList":
		return
	var payload: Dictionary = response.get("payload", {})
	var items: Array = payload.get("items", [])
	_sync_agents(items)


func _on_broadcast(event: String, payload: Dictionary, _source: String) -> void:
	match event:
		"agent_status_changed":
			var agent_id := str(payload.get("agent_id", ""))
			var status := str(payload.get("status", "sleeping"))
			if _spheres.has(agent_id):
				var sphere: AgentSphere = _spheres[agent_id]
				sphere.set_status(status)
		"agent_message":
			if communication_lines:
				communication_lines.on_agent_message(
					str(payload.get("from", "")),
					str(payload.get("to", "")),
					str(payload.get("message_id", "")),
				)


func _sync_agents(items: Array) -> void:
	var seen: Dictionary = {}
	for i in items.size():
		var item: Dictionary = items[i]
		var id := str(item.get("id", ""))
		if id.is_empty():
			continue
		seen[id] = true
		var name := str(item.get("name", id))
		var role := str(item.get("role", "assistant"))
		var status := str(item.get("status", "sleeping"))
		if _spheres.has(id):
			var existing: AgentSphere = _spheres[id]
			existing.configure(id, name, role, status)
		else:
			var sphere := AgentSphere.new()
			sphere.configure(id, name, role, status)
			var angle := (float(i) / maxf(1.0, float(items.size()))) * TAU
			sphere.position = Vector3(cos(angle) * ORBIT_RADIUS, 0.4, sin(angle) * ORBIT_RADIUS)
			var body := StaticBody3D.new()
			var col := CollisionShape3D.new()
			var shape := SphereShape3D.new()
			shape.radius = 0.4
			col.shape = shape
			body.add_child(col)
			sphere.add_child(body)
			add_child(sphere)
			_spheres[id] = sphere
			if communication_lines:
				communication_lines.register_agent(id, sphere)
	for id in _spheres.keys():
		if not seen.has(id):
			var old: AgentSphere = _spheres[id]
			if communication_lines:
				communication_lines.unregister_agent(id)
			old.queue_free()
			_spheres.erase(id)


func get_sphere(agent_id: String) -> AgentSphere:
	return _spheres.get(agent_id)