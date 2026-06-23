class_name MonitoringPanel
extends DockPanel
## Panneau Monitoring — activité, connexion, latence (Phase 20).

@onready var _bar: ProgressBar = %ActivityBar
@onready var _label: Label = %ActivityLabel
@onready var _status: Label = %StatusLabel
@onready var _latency: Label = %LatencyLabel
@onready var _mapper: Label = %MapperLabel

var _display_value := 0.0


func _ready() -> void:
	panel_id = "monitoring"
	panel_title = "Monitoring"
	super._ready()


func _process(delta: float) -> void:
	if _bar:
		_bar.value = lerpf(_bar.value, _display_value, delta * 8.0)


func update_activity(intensity: float) -> void:
	_display_value = ActivityMapper.clamp_intensity(intensity) * 100.0
	var pct := int(_display_value)
	_label.text = "Activité : %d %%" % pct


func update_mapper_activity(level: float) -> void:
	if _mapper:
		_mapper.text = "Niveau mapper : %.2f / 3" % clampf(level, 0.0, 3.0)


func update_sphere_fps(fps: float, tier: String) -> void:
	if _mapper:
		_mapper.text = "Sphère : %.0f FPS · qualité %s" % [fps, tier]


func update_latency(rtt_ms: float) -> void:
	if not _latency:
		return
	if rtt_ms < 0.0:
		_latency.text = "Latence : —"
		_latency.modulate = Color(0.7, 0.7, 0.75)
		return
	_latency.text = "Latence : %d ms" % int(rtt_ms)
	if rtt_ms < 80.0:
		_latency.modulate = Color(0.4, 0.9, 0.5)
	elif rtt_ms < 200.0:
		_latency.modulate = Color(0.85, 0.8, 0.35)
	else:
		_latency.modulate = Color(0.95, 0.45, 0.35)


func set_connection_status(connected: bool, detail: String) -> void:
	_status.text = ("● " if connected else "○ ") + detail
	_apply_status_color(connected, false)
	if not connected and _latency:
		_latency.text = "Latence : hors ligne"
		_latency.modulate = Color(0.9, 0.5, 0.4)


func set_degraded_mode(active: bool) -> void:
	_apply_status_color(not active, active)


func _apply_status_color(connected: bool, degraded: bool) -> void:
	if degraded:
		_status.modulate = Color(0.95, 0.55, 0.25)
	elif connected:
		_status.modulate = Color(0.4, 0.9, 0.5)
	else:
		_status.modulate = Color(0.9, 0.5, 0.4)