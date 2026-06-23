extends Node
class_name TerritoryManager
## Hub fenêtre principale — sphère neurale premium, panneaux, événements Phase 24.

@export var neural_brain: AINeuralBrainSphere
@export var sphere_performance: Node
@export var monitoring_panel: MonitoringPanel
@export var chat_panel: ChatPanel
@export var memory_list_panel: MemoryListPanel
@export var graph_panel: GraphPanel
@export var dock_layout: DockLayout

var _daemon: TerritoryDaemonClient
var _connected := false


func _ready() -> void:
	_daemon = TerritoryDaemonClient.resolve(self)
	if dock_layout:
		dock_layout.restore()

	if _daemon:
		_daemon.activity_changed.connect(_on_activity_changed)
		_daemon.brain_pulse_requested.connect(_on_brain_pulse)
		_daemon.connection_state_changed.connect(_on_connection_state)
		_daemon.latency_updated.connect(_on_latency_updated)

	if monitoring_panel:
		VisualEventMapper.activity_level_changed.connect(monitoring_panel.update_mapper_activity)
		if sphere_performance and sphere_performance.has_signal("fps_updated"):
			sphere_performance.fps_updated.connect(_on_fps_updated)

	if memory_list_panel and graph_panel:
		memory_list_panel.memory_selected.connect(graph_panel.highlight_memory)
		graph_panel.hub_selected.connect(memory_list_panel.focus_memory)


func _exit_tree() -> void:
	if dock_layout:
		dock_layout.save()


func _on_brain_pulse(boost: float, duration: float) -> void:
	if neural_brain:
		neural_brain.pulse_activity(boost, duration)


func _on_activity_changed(intensity: float) -> void:
	if neural_brain:
		neural_brain.update_brain_activity(intensity)
	if monitoring_panel:
		monitoring_panel.update_activity(intensity)


func _on_connection_state(connected: bool, detail: String) -> void:
	_connected = connected
	if neural_brain:
		neural_brain.set_degraded_mode(not connected)
	VisualEventMapper.set_degraded_mode(not connected)
	if monitoring_panel:
		monitoring_panel.set_connection_status(connected, detail)
		monitoring_panel.set_degraded_mode(not connected)


func _on_latency_updated(rtt_ms: float) -> void:
	if monitoring_panel:
		monitoring_panel.update_latency(rtt_ms)


func _on_fps_updated(fps: float, tier: String) -> void:
	if monitoring_panel and monitoring_panel.has_method("update_sphere_fps"):
		monitoring_panel.update_sphere_fps(fps, tier)