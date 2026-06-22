@tool
extends EditorPlugin

func _enter_tree() -> void:
	add_custom_type(
		"ForceGraph",
		"Control",
		preload("res://addons/force_directed_graph/force_graph.gd"),
		null
	)

func _exit_tree() -> void:
	remove_custom_type("ForceGraph")