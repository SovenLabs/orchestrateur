extends PanelContainer
## Panneau Monitoring dockable — intensité temps réel.

@onready var _bar: ProgressBar = %ActivityBar
@onready var _label: Label = %ActivityLabel
@onready var _status: Label = %StatusLabel


func _ready() -> void:
	if DaemonClient:
		DaemonClient.activity_changed.connect(update_activity)
		DaemonClient.connection_state_changed.connect(_on_connection_changed)


func update_activity(intensity: float) -> void:
	var pct := int(clampf(intensity, 0.0, 1.0) * 100.0)
	_bar.value = pct
	_label.text = "Activité : %d %%" % pct


func _on_connection_changed(connected: bool, detail: String) -> void:
	_status.text = ("● " if connected else "○ ") + detail
	_status.modulate = Color(0.4, 0.9, 0.5) if connected else Color(0.9, 0.5, 0.4)