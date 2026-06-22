class_name ChatPanel
extends DockPanel
## Panneau Chat — envoi/réception via WebSocket (fenêtre locale).

@onready var _input: TextEdit = %ChatInput
@onready var _reply: RichTextLabel = %ChatReply
@onready var _send_btn: Button = %SendButton

var _daemon: DaemonClient
var _pending_rid := ""


func _ready() -> void:
	panel_id = "chat"
	panel_title = "Chat"
	super._ready()
	_daemon = DaemonClient.resolve(self)
	_send_btn.pressed.connect(_on_send)
	_input.gui_input.connect(_on_input_gui)
	if _daemon:
		_daemon.command_completed.connect(_on_command_completed)
		_daemon.broadcast_received.connect(_on_broadcast)


func _on_input_gui(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and event.keycode == KEY_ENTER:
		if event.shift_pressed:
			return
		_on_send()
		get_viewport().set_input_as_handled()


func _on_send() -> void:
	if not _daemon:
		return
	var text := _input.text.strip_edges()
	if text.is_empty():
		return
	_reply.append_text("\n[color=#88aaff]Vous :[/color] %s\n" % text)
	_input.text = ""
	_pending_rid = _daemon.execute_chat(text)


func _on_command_completed(request_id: String, response: Dictionary) -> void:
	if request_id != _pending_rid:
		return
	_pending_rid = ""
	_append_reply(response)


func _on_broadcast(event: String, payload: Dictionary, _source: String) -> void:
	if event != "chat_reply":
		return
	_append_reply({"response": "ChatReply", "payload": payload})


func _append_reply(response: Dictionary) -> void:
	var kind: String = str(response.get("response", ""))
	if kind == "ChatReply":
		var reply: String = str(response.get("payload", {}).get("reply", ""))
		_reply.append_text("[color=#aaffaa]Agent :[/color] %s\n" % reply)
	elif kind == "Error":
		var msg: String = str(response.get("payload", {}).get("message", "Erreur"))
		_reply.append_text("[color=#ff8888]Erreur :[/color] %s\n" % msg)