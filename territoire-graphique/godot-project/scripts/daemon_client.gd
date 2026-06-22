extends Node
class_name TerritoryDaemonClient
## Client WebSocket par fenêtre — hub Command/Response + broadcast territorial.

signal activity_changed(intensity: float)
signal connection_state_changed(connected: bool, detail: String)
signal command_completed(request_id: String, response: Dictionary)
signal brain_pulse_requested(boost: float, duration: float)
signal broadcast_received(event: String, payload: Dictionary, source_session_id: String)
signal latency_updated(rtt_ms: float)

const WS_URL := "ws://127.0.0.1:28790/ws"
const HEALTH_URL := "http://127.0.0.1:28790/health"
const POLL_SECS := 2.0
const PING_SECS := 25.0
const RECONNECT_BASE := 1.0
const RECONNECT_MAX := 30.0

var session_id := ""
var territory_session_id := ""

var _peer := WebSocketPeer.new()
var _http := HTTPRequest.new()
var _connected := false
var _authenticated := false
var _request_id := 0
var _pending_health := false
var _poll_timer := 0.0
var _ping_timer := 0.0
var _fallback_elapsed := 0.0
var _use_fallback := false
var _token := "dev"
var _reconnect_delay := RECONNECT_BASE
var _reconnect_timer := 0.0
var _window_kind := "main"
var _panels: Array = []
var _ping_nonce := 0
var _ping_sent_at := 0.0
var _last_rtt_ms := -1.0


func _ready() -> void:
	_token = OS.get_environment("ORCHESTRATEUR_DAEMON_TOKEN")
	if _token.is_empty():
		_token = "dev"
	_http.request_completed.connect(_on_http_health)
	add_child(_http)
	add_to_group("territory_daemon")
	_connect_ws()


static func resolve(from: Node) -> TerritoryDaemonClient:
	if not from:
		return null
	return from.get_tree().get_first_node_in_group("territory_daemon") as TerritoryDaemonClient


func configure_window(kind: String, panels: Array) -> void:
	_window_kind = kind
	_panels = panels.duplicate()
	if _authenticated:
		_send_connect()


func is_ready() -> bool:
	return _authenticated and not _use_fallback


func is_main_window() -> bool:
	return _window_kind == "main"


func execute(command: Dictionary) -> String:
	_request_id += 1
	var rid := str(_request_id)
	if not is_ready():
		command_completed.emit(rid, {
			"response": "Error",
			"payload": {"kind": "offline", "message": "Daemon non connecté"},
		})
		return rid
	var msg := {"type": "execute", "request_id": rid, "command": command}
	_peer.send_text(JSON.stringify(msg))
	return rid


func execute_chat(message: String) -> String:
	if is_main_window():
		brain_pulse_requested.emit(0.35, 0.4)
	return execute({"command": "Chat", "payload": {"message": message}})


func execute_list(filter: String = "", offset: int = 0, limit: int = 100) -> String:
	var payload := {"filter": null, "offset": offset, "limit": limit}
	if not filter.is_empty():
		payload["filter"] = filter
	return execute({"command": "List", "payload": payload})


func execute_search(query: String, limit: int = 30) -> String:
	return execute({"command": "Search", "payload": {"query": query, "limit": limit}})


func execute_get_memory(id: String) -> String:
	return execute({"command": "GetMemory", "payload": {"id": id}})


func execute_graph() -> String:
	return execute({"command": "Graph"})


func _connect_ws() -> void:
	var err := _peer.connect_to_url(WS_URL)
	if err != OK:
		_schedule_reconnect("WebSocket indisponible (%s)" % error_string(err))
		return
	_use_fallback = false
	_reconnect_timer = 0.0


func _schedule_reconnect(reason: String) -> void:
	_use_fallback = true
	_connected = false
	_authenticated = false
	connection_state_changed.emit(false, reason)
	_reconnect_timer = _reconnect_delay
	_reconnect_delay = minf(_reconnect_delay * 2.0, RECONNECT_MAX)


func _process(delta: float) -> void:
	_fallback_elapsed += delta

	if _use_fallback:
		_poll_http_health()
		var idle := ActivityMapper.fallback_idle(_fallback_elapsed)
		activity_changed.emit(idle)
		if is_main_window():
			VisualEventMapper.emit_idle_breathing(_fallback_elapsed, false)
			VisualEventMapper.set_degraded_mode(true)
		_reconnect_timer -= delta
		if _reconnect_timer <= 0.0:
			_peer = WebSocketPeer.new()
			_connect_ws()
		return

	_peer.poll()
	var state := _peer.get_ready_state()
	match state:
		WebSocketPeer.STATE_OPEN:
			if not _connected:
				_connected = true
				_reconnect_delay = RECONNECT_BASE
				_send_connect()
			while _peer.get_available_packet_count() > 0:
				_handle_packet(_peer.get_packet().get_string_from_utf8())
			_poll_timer += delta
			_ping_timer += delta
			if _authenticated and _poll_timer >= POLL_SECS:
				_poll_timer = 0.0
				_request_health()
			if _authenticated and _ping_timer >= PING_SECS:
				_ping_timer = 0.0
				_send_ping()
		WebSocketPeer.STATE_CLOSING, WebSocketPeer.STATE_CLOSED:
			if _connected:
				_schedule_reconnect("Connexion WS fermée")
			else:
				_schedule_reconnect("Daemon non joignable")
			_peer = WebSocketPeer.new()


func _send_connect() -> void:
	var payload := {
		"type": "connect",
		"token": _token,
		"client": {
			"window_kind": _window_kind,
			"panels": _panels,
			"subscriptions": _default_subscriptions(),
		},
	}
	_peer.send_text(JSON.stringify(payload))


func _default_subscriptions() -> Array:
	if _window_kind == "main":
		return [
			"activity", "visual", "memories", "graph", "chat", "brain_pulse",
			"memory_assimilated", "tool_call", "vector_search",
			"system_error", "degraded_mode",
		]
	var subs: Array = ["activity"]
	for panel_id in _panels:
		match str(panel_id):
			"chat":
				subs.append("chat")
				subs.append("memories")
			"memory":
				subs.append("memories")
			"graph":
				subs.append("graph")
				subs.append("memories")
			"monitoring":
				subs.append("activity")
	return subs


func _send_ping() -> void:
	_ping_nonce += 1
	_ping_sent_at = Time.get_ticks_msec()
	_peer.send_text(JSON.stringify({"type": "ping", "nonce": _ping_nonce}))


func _request_health() -> void:
	execute({"command": "HealthCheck"})


func _handle_packet(text: String) -> void:
	var data = JSON.parse_string(text)
	if typeof(data) != TYPE_DICTIONARY:
		return
	match data.get("type", ""):
		"connect_ok":
			_authenticated = true
			session_id = str(data.get("session_id", ""))
			territory_session_id = str(data.get("territory_session_id", ""))
			connection_state_changed.emit(
				true,
				"Connecté v%s · %s" % [data.get("version", "?"), session_id.substr(0, 8)]
			)
			VisualEventMapper.set_degraded_mode(false)
			_bootstrap_data()
		"result":
			_dispatch_result(data)
		"broadcast":
			_handle_broadcast(data)
		"pong":
			if _ping_sent_at > 0.0:
				_last_rtt_ms = (Time.get_ticks_msec() - _ping_sent_at)
				latency_updated.emit(_last_rtt_ms)
				_ping_sent_at = 0.0
		"error":
			push_warning("Daemon: %s" % data.get("message", "erreur"))


func _bootstrap_data() -> void:
	_request_health()
	if _window_kind == "main" or "memory" in _panels:
		execute_list()
	if _window_kind == "main" or "graph" in _panels:
		execute_graph()


func _handle_broadcast(data: Dictionary) -> void:
	var event := str(data.get("event", ""))
	var payload: Dictionary = data.get("payload", {})
	var source := str(data.get("source_session_id", ""))
	if source == session_id:
		return
	broadcast_received.emit(event, payload, source)
	_route_visual_event(event, payload)
	match event:
		"memories_changed", "memory_assimilated":
			if _window_kind == "main" or "memory" in _panels:
				execute_list()
		"graph_changed":
			if _window_kind == "main" or "graph" in _panels:
				execute_graph()
		"brain_pulse":
			if is_main_window():
				brain_pulse_requested.emit(
					float(payload.get("boost", 0.4)),
					float(payload.get("duration", 0.5)),
				)
		"chat_reply":
			pass
		"degraded_mode", "system_error":
			if monitoring_panel_visible():
				connection_state_changed.emit(false, str(payload.get("message", "Mode dégradé")))


func _dispatch_result(data: Dictionary) -> void:
	var response: Dictionary = data.get("response", {})
	var rid: String = str(data.get("request_id", ""))
	var kind: String = str(response.get("response", ""))

	if kind == "Health":
		var payload: Dictionary = response.get("payload", {})
		var status := str(payload.get("status", "unknown"))
		var intensity := ActivityMapper.clamp_intensity(
			ActivityMapper.from_health(
				status,
				bool(payload.get("llm_available", false)),
				bool(payload.get("embedding_available", false)),
			)
		)
		activity_changed.emit(intensity)
		if is_main_window():
			VisualEventMapper.map_backend_event(
				"agent_activity",
				{"level": intensity * 3.0},
			)
			if status == "degraded":
				VisualEventMapper.map_backend_event("degraded_mode", payload)

	if kind == "ChatReply":
		var tools: Array = response.get("payload", {}).get("tools_invoked", [])
		if is_main_window():
			if tools.size() > 0:
				_route_visual_event("tool_call", {
					"tools": tools,
					"tool_name": str(tools[0]),
				})
			else:
				_route_visual_event("brain_pulse", {"boost": 0.5, "duration": 0.6, "kind": "chat"})
		if response.get("payload", {}).get("auto_assimilated"):
			if _window_kind == "main" or "memory" in _panels:
				execute_list()
			if _window_kind == "main" or "graph" in _panels:
				execute_graph()

	if kind == "Assimilated":
		if is_main_window():
			_route_visual_event("memory_assimilated", response.get("payload", {}))
		if _window_kind == "main" or "memory" in _panels:
			execute_list()
		if _window_kind == "main" or "graph" in _panels:
			execute_graph()

	if kind == "SearchResults":
		if is_main_window():
			_route_visual_event("vector_search", {
				"hit_count": response.get("payload", {}).get("items", []).size(),
			})

	if kind == "Error":
		if is_main_window():
			_route_visual_event(
				"degraded_mode" if str(response.get("payload", {}).get("kind", "")) == "degraded" else "system_error",
				response.get("payload", {}),
			)

	command_completed.emit(rid, response)


func _route_visual_event(event: String, payload: Dictionary) -> void:
	if not is_main_window():
		return
	VisualEventMapper.map_backend_event(event, payload)


func get_latency_ms() -> float:
	return _last_rtt_ms


func monitoring_panel_visible() -> bool:
	return _window_kind == "main" or "monitoring" in _panels


func _poll_http_health() -> void:
	if _http.get_http_client_status() != HTTPClient.STATUS_DISCONNECTED or _pending_health:
		return
	_pending_health = true
	_http.request(HEALTH_URL)


func _on_http_health(_result: int, _code: int, _headers: PackedStringArray, body: PackedByteArray) -> void:
	_pending_health = false
	if _code != 200:
		return
	var data = JSON.parse_string(body.get_string_from_utf8())
	if typeof(data) == TYPE_DICTIONARY:
		var raw := 0.45 if data.get("status") == "ok" else 0.3
		activity_changed.emit(ActivityMapper.clamp_intensity(raw))
		connection_state_changed.emit(true, "HTTP health (fallback)")