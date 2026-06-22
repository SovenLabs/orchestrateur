class_name DockLayout
extends Node
## Sauvegarde / restauration des séparateurs de panneaux (préparation Phase 18).

const LAYOUT_PATH := "user://territory_layout.json"

@export var h_split: HSplitContainer
@export var left_v: VSplitContainer
@export var right_v: VSplitContainer


func save() -> void:
	if not h_split:
		return
	var data := {
		"h_split": h_split.split_offset,
		"left_v": left_v.split_offset if left_v else 200,
		"right_v": right_v.split_offset if right_v else 220,
	}
	var f := FileAccess.open(LAYOUT_PATH, FileAccess.WRITE)
	if f:
		f.store_string(JSON.stringify(data))


func restore() -> void:
	if not FileAccess.file_exists(LAYOUT_PATH):
		return
	var f := FileAccess.open(LAYOUT_PATH, FileAccess.READ)
	if not f:
		return
	var data = JSON.parse_string(f.get_as_text())
	if typeof(data) != TYPE_DICTIONARY:
		return
	if h_split and data.has("h_split"):
		h_split.split_offset = int(data["h_split"])
	if left_v and data.has("left_v"):
		left_v.split_offset = int(data["left_v"])
	if right_v and data.has("right_v"):
		right_v.split_offset = int(data["right_v"])