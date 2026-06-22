class_name MonitoringPanel
extends DockPanel
## Panneau Monitoring — barre lissée + statut connexion.

@onready var _bar: ProgressBar = %ActivityBar
@onready var _label: Label = %ActivityLabel
@onready var _status: Label = %StatusLabel

var _display_value := 0.0


func _ready() -> void:
	panel_title = "Monitoring"
	super._ready()


func _process(delta: float) -> void:
	if _bar:
		_bar.value = lerpf(_bar.value, _display_value, delta * 8.0)


func update_activity(intensity: float) -> void:
	_display_value = ActivityMapper.clamp_intensity(intensity) * 100.0
	var pct := int(_display_value)
	_label.text = "Activité : %d %%" % pct


func set_connection_status(connected: bool, detail: String) -> void:
	_status.text = ("● " if connected else "○ ") + detail
	_status.modulate = Color(0.4, 0.9, 0.5) if connected else Color(0.9, 0.5, 0.4)