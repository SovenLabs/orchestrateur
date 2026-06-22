class_name ChatPanel
extends DockPanel
## Panneau Chat — envoi/réception via WebSocket.

@onready var _input: TextEdit = %ChatInput
@onready var _reply: RichTextLabel = %ChatReply
@onready var _send_btn: Button = %SendButton

var _pending_rid := ""


func _ready() -> void:
	panel_title = "Chat"
	super._ready()
	_send_btn.pressed.connect(_on_send)
	_input.gui_input.connect(_on_input_gui)
	DaemonClient.command_completed.connect(_on_command_completed)


func _on_input_gui(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and event.keycode == KEY_ENTER:
		if event.shift_pressed:
			return
		_on_send()
		get_viewport().set_input_as_handled()


func _on_send() -> void:
	var text := _input.text.strip_edges()
	if text.is_empty():
		return
	_reply.append_text("\n[color=#88aaff]Vous :[/color] %s\n" % text)
	_input.text = ""
	_pending_rid = DaemonClient.execute_chat(text)


func _on_command_completed(request_id: String, response: Dictionary) -> void:
	if request_id != _pending_rid:
		return
	_pending_rid = ""
	var kind: String = str(response.get("response", ""))
	if kind == "ChatReply":
		var reply: String = str(response.get("payload", {}).get("reply", ""))
		_reply.append_text("[color=#aaffaa]Agent :[/color] %s\n" % reply)
	elif kind == "Error":
		var msg: String = str(response.get("payload", {}).get("message", "Erreur"))
		_reply.append_text("[color=#ff8888]Erreur :[/color] %s\n" % msg)