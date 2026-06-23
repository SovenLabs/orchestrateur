extends Node
## Gestionnaire multi-fenêtrage — une Boule, extensions détachables (Phase 20).

const EXTENSION_SCENE_PATH := "res://scenes/ExtensionTerritory.tscn"
const SPHERE_DEDICATED_SCENE := "res://scenes/SphereDedicated.tscn"

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
		win.close_requested.connect(_on_main_close_requested)


func open_sphere_dedicated() -> void:
	if _extensions.has("sphere"):
		var existing: Window = _extensions["sphere"]
		if is_instance_valid(existing):
			existing.grab_focus()
			return
		_extensions.erase("sphere")

	var packed: PackedScene = load(SPHERE_DEDICATED_SCENE)
	if not packed:
		push_error("Scène sphère dédiée introuvable")
		return

	var win := Window.new()
	win.title = "Orchestrateur — Boule de Pixels Vivante"
	win.size = Vector2i(1280, 800)
	win.min_size = Vector2i(800, 600)
	win.close_requested.connect(_on_sphere_close_requested)

	var content: Node3D = packed.instantiate()
	win.add_child(content)
	get_tree().root.add_child(win)
	win.tree_exited.connect(_on_extension_tree_exited.bind("sphere"))
	win.show()
	_extensions["sphere"] = win


func _on_sphere_close_requested() -> void:
	_close_extension("sphere")


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
	win.size = Vector2i(960, 720)
	win.min_size = Vector2i(480, 360)
	win.close_requested.connect(_on_extension_close_requested.bind(panel_id))

	var content: Control = packed.instantiate()
	if content.has_method("configure"):
		content.configure(panel_id)
	win.add_child(content)
	get_tree().root.add_child(win)
	TerritoryTheme.apply_to(content)
	win.tree_exited.connect(_on_extension_tree_exited.bind(panel_id))
	win.show()
	_extensions[panel_id] = win


func is_extension_open(panel_id: String) -> bool:
	return _extensions.has(panel_id) and is_instance_valid(_extensions[panel_id])


func close_all_extensions() -> void:
	for panel_id in _extensions.keys():
		_close_extension(str(panel_id))


func _on_extension_close_requested(panel_id: String) -> void:
	_close_extension(panel_id)


func _close_extension(panel_id: String) -> void:
	var win: Window = _extensions.get(panel_id)
	if win and is_instance_valid(win):
		var content := win.get_child(0)
		if content and content.has_method("cleanup"):
			content.cleanup()
		win.hide()
		win.queue_free()
	_extensions.erase(panel_id)


func _on_extension_tree_exited(panel_id: String) -> void:
	_extensions.erase(panel_id)


func _on_main_close_requested() -> void:
	close_all_extensions()
	if main_window and is_instance_valid(main_window):
		main_window.queue_free()
	main_window = null