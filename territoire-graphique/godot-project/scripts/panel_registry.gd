extends RefCounted
class_name PanelRegistry
## Registre des panneaux détachables — scènes et métadonnées.

const PANEL_SCENES := {
	"chat": "res://scenes/ChatPanel.tscn",
	"memory": "res://scenes/MemoryListPanel.tscn",
	"graph": "res://scenes/GraphPanel.tscn",
	"monitoring": "res://scenes/MonitoringPanel.tscn",
}

const PANEL_TITLES := {
	"chat": "Chat",
	"memory": "Mémoires",
	"graph": "Graphe",
	"monitoring": "Monitoring",
}


static func scene_for(panel_id: String) -> String:
	return PANEL_SCENES.get(panel_id, "")


static func title_for(panel_id: String) -> String:
	return PANEL_TITLES.get(panel_id, "Extension")