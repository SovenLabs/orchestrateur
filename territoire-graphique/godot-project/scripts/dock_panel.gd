class_name DockPanel
extends PanelContainer
## Panneau dockable — titre, détachement Phase 18.

signal detach_requested

@export var panel_title := "Panneau"
@export var panel_id := ""

@onready var _title_label: Label = %TitleLabel
@onready var _detach_btn: Button = %DetachButton


func _ready() -> void:
	if _title_label:
		_title_label.text = panel_title
	if _detach_btn:
		_detach_btn.pressed.connect(_on_detach_pressed)
	TerritoryTheme.apply_to(self)


func get_panel_id() -> String:
	return panel_id


func _on_detach_pressed() -> void:
	if panel_id.is_empty():
		push_warning("panel_id non défini — détachement impossible")
		return
	detach_requested.emit()
	WindowManager.open_extension(panel_id)