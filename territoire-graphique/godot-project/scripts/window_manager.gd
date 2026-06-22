extends Node
## Gestionnaire multi-fenêtrage — une Boule, extensions détachables.

const EXTENSION_SCENE_PATH := "res://scenes/ExtensionTerritory.tscn"

var main_window: Window = null
var _extensions: Dictionary = {}


func register_main(root: Node) -> void:
	var win := root.get_window()
	if main_window and is_instance_valid(main_window):
		push_warning("Fenêtre principale déjà enregistrée")
		return
	main_window = win
	if win:
		win.title = "Territoire Graphique — Cortex"


func open_extension(panel_id: String) -> void:
	if panel_id.is_empty():
		return
	if _extensions.has(panel_id):
		var existing: Window = _extensions[panel_id]
		if is_instance_valid(existing):
			existing.grab_focus()
			return
		_extensions.erase(panel_id)

	var packed: PackedScene = load(EXTENSION_SCENE_PATH)
	if not packed:
		push_error("Scène extension introuvable : %s" % EXTENSION_SCENE_PATH)
		return

	var win := Window.new()
	win.title = "Territoire — %s" % PanelRegistry.title_for(panel_id)
	win.size = Vector2i(720, 560)
	win.min_size = Vector2i(420, 320)
	win.close_requested.connect(_on_extension_close_requested.bind(panel_id))

	var content: Control = packed.instantiate()
	if content.has_method("configure"):
		content.configure(panel_id)
	win.add_child(content)
	get_tree().root.add_child(win)
	TerritoryTheme.apply_to(content)
	win.show()
	_extensions[panel_id] = win


func is_extension_open(panel_id: String) -> bool:
	return _extensions.has(panel_id) and is_instance_valid(_extensions[panel_id])


func _on_extension_close_requested(panel_id: String) -> void:
	var win: Window = _extensions.get(panel_id)
	if win and is_instance_valid(win):
		win.queue_free()
	_extensions.erase(panel_id)