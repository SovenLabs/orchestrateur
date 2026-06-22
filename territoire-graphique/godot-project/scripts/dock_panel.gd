class_name DockPanel
extends PanelContainer
## Panneau dockable de base — titre + signal détachement (Phase 18).

signal detach_requested

@export var panel_title := "Panneau"

@onready var _title_label: Label = %TitleLabel
@onready var _detach_btn: Button = %DetachButton


func _ready() -> void:
	if _title_label:
		_title_label.text = panel_title
	if _detach_btn:
		_detach_btn.pressed.connect(_on_detach_pressed)


func _on_detach_pressed() -> void:
	detach_requested.emit()