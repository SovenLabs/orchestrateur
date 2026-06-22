class_name DockLayout
extends Node
## Layout responsive — ratios proportionnels au redimensionnement fenêtre.

const LAYOUT_PATH := "user://territory_layout.json"
const LEGACY_WIDTH := 1280.0
const DESIGN_WIDTH := 1920.0

const DEFAULT_LEFT_RATIO := 340.0 / DESIGN_WIDTH
const DEFAULT_RIGHT_RATIO := 380.0 / DESIGN_WIDTH
const DEFAULT_LEFT_V_RATIO := 0.48
const DEFAULT_RIGHT_V_RATIO := 0.52

const MIN_LEFT_W := 260
const MIN_RIGHT_W := 280
const MIN_CENTER_W := 320

@export var left_column: Control
@export var right_column: Control
@export var left_v: VSplitContainer
@export var right_v: VSplitContainer

var _left_ratio := DEFAULT_LEFT_RATIO
var _right_ratio := DEFAULT_RIGHT_RATIO
var _left_v_ratio := DEFAULT_LEFT_V_RATIO
var _right_v_ratio := DEFAULT_RIGHT_V_RATIO
var _applying := false


func _ready() -> void:
	restore()
	var vp := get_viewport()
	if vp and not vp.size_changed.is_connected(_on_viewport_resized):
		vp.size_changed.connect(_on_viewport_resized)
	if left_v and not left_v.dragged.is_connected(_on_left_v_dragged):
		left_v.dragged.connect(_on_left_v_dragged)
	if right_v and not right_v.dragged.is_connected(_on_right_v_dragged):
		right_v.dragged.connect(_on_right_v_dragged)
	call_deferred("apply_layout")


func apply_layout() -> void:
	if _applying:
		return
	_applying = true

	var vp_size := get_viewport().get_visible_rect().size
	var total_w := vp_size.x
	var total_h := maxf(vp_size.y, 1.0)

	if total_w > 0.0:
		var left_w := _clamp_side_width(int(total_w * _left_ratio), MIN_LEFT_W, total_w)
		var right_w := _clamp_side_width(int(total_w * _right_ratio), MIN_RIGHT_W, total_w - left_w - MIN_CENTER_W)
		if left_w + right_w + MIN_CENTER_W > int(total_w):
			var overflow := left_w + right_w + MIN_CENTER_W - int(total_w)
			right_w = maxi(MIN_RIGHT_W, right_w - overflow)

		if left_column:
			left_column.offset_right = left_w
		if right_column:
			right_column.offset_left = -right_w
			right_column.offset_right = 0

	if left_v:
		left_v.split_offset = maxi(160, int(total_h * _left_v_ratio))
	if right_v:
		right_v.split_offset = maxi(160, int(total_h * _right_v_ratio))

	_applying = false


func save() -> void:
	_capture_ratios_from_controls()
	var data := {
		"version": 2,
		"left_ratio": _left_ratio,
		"right_ratio": _right_ratio,
		"left_v_ratio": _left_v_ratio,
		"right_v_ratio": _right_v_ratio,
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

	if int(data.get("version", 1)) >= 2:
		_left_ratio = float(data.get("left_ratio", DEFAULT_LEFT_RATIO))
		_right_ratio = float(data.get("right_ratio", DEFAULT_RIGHT_RATIO))
		_left_v_ratio = float(data.get("left_v_ratio", DEFAULT_LEFT_V_RATIO))
		_right_v_ratio = float(data.get("right_v_ratio", DEFAULT_RIGHT_V_RATIO))
	else:
		_migrate_legacy_layout(data)


func _migrate_legacy_layout(data: Dictionary) -> void:
	if data.has("h_split"):
		_left_ratio = clampf(float(data["h_split"]) / LEGACY_WIDTH, 0.12, 0.35)
	if data.has("right_v"):
		_right_v_ratio = clampf(float(data["right_v"]) / 800.0, 0.25, 0.75)
	if data.has("left_v"):
		_left_v_ratio = clampf(float(data["left_v"]) / 800.0, 0.25, 0.75)
	_right_ratio = DEFAULT_RIGHT_RATIO


func _capture_ratios_from_controls() -> void:
	var vp_w := get_viewport().get_visible_rect().size.x
	if vp_w <= 0.0:
		return
	if left_column:
		_left_ratio = clampf(left_column.size.x / vp_w, 0.12, 0.35)
	if right_column:
		_right_ratio = clampf(right_column.size.x / vp_w, 0.12, 0.35)
	var vp_h := get_viewport().get_visible_rect().size.y
	if vp_h > 0.0:
		if left_v:
			_left_v_ratio = clampf(left_v.split_offset / vp_h, 0.2, 0.8)
		if right_v:
			_right_v_ratio = clampf(right_v.split_offset / vp_h, 0.2, 0.8)


func _clamp_side_width(desired: int, minimum: int, max_available: float) -> int:
	return clampi(desired, minimum, int(max_available))


func _on_viewport_resized() -> void:
	apply_layout()


func _on_left_v_dragged(_offset: int) -> void:
	if _applying:
		return
	var vp_h := get_viewport().get_visible_rect().size.y
	if vp_h > 0.0 and left_v:
		_left_v_ratio = clampf(left_v.split_offset / vp_h, 0.2, 0.8)


func _on_right_v_dragged(_offset: int) -> void:
	if _applying:
		return
	var vp_h := get_viewport().get_visible_rect().size.y
	if vp_h > 0.0 and right_v:
		_right_v_ratio = clampf(right_v.split_offset / vp_h, 0.2, 0.8)