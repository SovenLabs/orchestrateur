extends Node
## Thème centralisé — cohérence visuelle entre fenêtres principale et extensions.

const BG_COLOR := Color(0.04, 0.06, 0.11, 0.92)
const PANEL_BG := Color(0.06, 0.08, 0.14, 0.95)
const ACCENT := Color(0.55, 0.82, 1.0)
const TEXT_MUTED := Color(0.65, 0.72, 0.8)
const FONT_SIZE_TITLE := 13
const FONT_SIZE_BODY := 12


func _ready() -> void:
	_apply_root_theme(get_tree().root)


func apply_to(node: Node) -> void:
	if not node:
		return
	if node is CanvasItem:
		(node as CanvasItem).modulate = Color.WHITE
	if node is Control:
		var ctrl := node as Control
		if ctrl is PanelContainer:
			var style := StyleBoxFlat.new()
			style.bg_color = PANEL_BG
			style.set_corner_radius_all(6)
			style.set_content_margin_all(4)
			ctrl.add_theme_stylebox_override("panel", style)


func _apply_root_theme(root: Node) -> void:
	if root is Window:
		var win := root as Window
		if win.transparent_bg:
			return