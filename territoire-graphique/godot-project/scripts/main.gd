extends Node3D
## Fenêtre principale — seule scène autorisée à héberger la Boule de Pixels.

@onready var _daemon: TerritoryDaemonClient = $DaemonClient


func _ready() -> void:
	WindowManager.register_main(self)
	if _daemon:
		_daemon.configure_window("main", ["chat", "memory", "graph", "monitoring"])