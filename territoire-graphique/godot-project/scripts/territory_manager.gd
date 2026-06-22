extends Node
class_name TerritoryManager
## Hub fenêtre principale — boule réactive, panneaux, événements visuels.

@export var brain_sphere: BrainSphere
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
		_daemon.visual_event.connect(_on_visual_event)

	if memory_list_panel and graph_panel:
		memory_list_panel.memory_selected.connect(graph_panel.highlight_memory)
		graph_panel.hub_selected.connect(memory_list_panel.focus_memory)


func _exit_tree() -> void:
	if dock_layout:
		dock_layout.save()


func _on_brain_pulse(boost: float, duration: float) -> void:
	if brain_sphere:
		brain_sphere.pulse_activity(boost, duration)


func _on_visual_event(effect: Dictionary) -> void:
	if brain_sphere and VisualEventMapper.should_apply(effect):
		brain_sphere.apply_visual_effect(effect)


func _on_activity_changed(intensity: float) -> void:
	if brain_sphere:
		brain_sphere.update_brain_activity(intensity)
	if monitoring_panel:
		monitoring_panel.update_activity(intensity)


func _on_connection_state(connected: bool, detail: String) -> void:
	_connected = connected
	if brain_sphere:
		brain_sphere.set_degraded_mode(not connected)
	if monitoring_panel:
		monitoring_panel.set_connection_status(connected, detail)
		monitoring_panel.set_degraded_mode(not connected)