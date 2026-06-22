class_name GraphPanel
extends DockPanel
## Panneau Graphe — hubs force-directed.

signal hub_selected(memory_id: String)

@onready var _graph: ForceGraph = %ForceGraph
@onready var _stats: Label = %GraphStats

var _daemon: DaemonClient
var _pending_rid := ""


func _ready() -> void:
	panel_id = "graph"
	panel_title = "Graphe"
	super._ready()
	_daemon = DaemonClient.resolve(self)
	if _graph:
		_graph.node_clicked.connect(_on_node_clicked)
	if _daemon:
		_daemon.command_completed.connect(_on_command_completed)
		_daemon.broadcast_received.connect(_on_broadcast)


func refresh_graph() -> void:
	if _daemon:
		_pending_rid = _daemon.execute_graph()


func highlight_memory(memory_id: String) -> void:
	if _graph:
		_graph.highlight_memory(memory_id)


func _on_broadcast(event: String, _payload: Dictionary, _source: String) -> void:
	if event == "graph_changed":
		refresh_graph()


func _on_command_completed(request_id: String, response: Dictionary) -> void:
	if request_id != _pending_rid:
		return
	_pending_rid = ""
	if str(response.get("response", "")) != "GraphSummary":
		return
	var payload: Dictionary = response.get("payload", {})
	var nodes: int = int(payload.get("node_count", 0))
	var edges: int = int(payload.get("edge_count", 0))
	_stats.text = "Nœuds: %d · Arêtes: %d" % [nodes, edges]
	var hubs: Array = payload.get("hubs", [])
	if _graph:
		_graph.set_hubs(hubs)


func _on_node_clicked(memory_id: String) -> void:
	hub_selected.emit(memory_id)