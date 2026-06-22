extends Control
class_name ExtensionTerritory
## Fenêtre d'extension du territoire — panneau unique, pas de Boule.

@onready var _daemon: DaemonClient = $DaemonClient
@onready var _slot: MarginContainer = %PanelSlot
@onready var _status: Label = %StatusLabel

var panel_id := ""


func configure(id: String) -> void:
	panel_id = id


func _ready() -> void:
	TerritoryTheme.apply_to(self)
	call_deferred("_mount")


func _mount() -> void:
	if panel_id.is_empty():
		_status.text = "Extension sans panneau"
		return
	var scene_path := PanelRegistry.scene_for(panel_id)
	if scene_path.is_empty():
		_status.text = "Panneau inconnu : %s" % panel_id
		return

	var packed: PackedScene = load(scene_path)
	if not packed:
		_status.text = "Scène introuvable"
		return

	var panel: Control = packed.instantiate()
	_slot.add_child(panel)
	_status.text = "Extension · %s · territoire partagé" % PanelRegistry.title_for(panel_id)

	if _daemon:
		_daemon.configure_window("extension", [panel_id])