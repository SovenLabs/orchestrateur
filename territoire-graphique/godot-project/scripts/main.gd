extends Node3D
## Scène racine Territoire Graphique — hub activité Phase 16.

@onready var _brain: Node3D = $BrainSphere
@onready var _monitoring: PanelContainer = $UI/MonitoringPanel

var _current_activity := 0.35


func _ready() -> void:
	print("Territoire Graphique v2 — shader + particules + WebSocket")
	if DaemonClient:
		DaemonClient.activity_changed.connect(_on_activity_changed)
		DaemonClient.connection_state_changed.connect(_on_connection_changed)


func _on_activity_changed(intensity: float) -> void:
	_current_activity = ActivityMapper.clamp_intensity(intensity)
	if _brain and _brain.has_method("update_brain_activity"):
		_brain.update_brain_activity(_current_activity)
	if _monitoring and _monitoring.has_method("update_activity"):
		_monitoring.update_activity(_current_activity)


func _on_connection_changed(connected: bool, detail: String) -> void:
	if _monitoring and _monitoring.has_method("set_connection_status"):
		_monitoring.set_connection_status(connected, detail)