extends Node
## Client WebSocket Option B — fallback HTTP /health puis GDExtension si chargée.

signal activity_changed(intensity: float)
signal connection_state_changed(connected: bool, detail: String)

const WS_URL := "ws://127.0.0.1:28790/ws"
const HEALTH_URL := "http://127.0.0.1:28790/health"
const POLL_SECS := 2.0

var _peer := WebSocketPeer.new()
var _http := HTTPRequest.new()
var _connected := false
var _authenticated := false
var _request_id := 0
var _pending_health := false
var _poll_timer := 0.0
var _fallback_elapsed := 0.0
var _use_fallback := false
var _token := "dev"
var _reconnect_cooldown := 0.0


func _ready() -> void:
	_token = OS.get_environment("ORCHESTRATEUR_DAEMON_TOKEN")
	if _token.is_empty():
		_token = "dev"
	_http.request_completed.connect(_on_http_health)
	add_child(_http)
	_try_gdextension_probe()
	_connect_ws()


func _try_gdextension_probe() -> void:
	if ClassDB.class_exists("TerritoireBridge"):
		var bridge = ClassDB.instantiate("TerritoireBridge")
		if bridge and bridge.has_method("default_ws_url"):
			print("GDExtension TerritoireBridge disponible — ", bridge.call("default_ws_url"))


func _connect_ws() -> void:
	var err := _peer.connect_to_url(WS_URL)
	if err != OK:
		_enable_fallback("WebSocket indisponible (%s)" % error_string(err))
		return
	_use_fallback = false
	print("Connexion daemon… ", WS_URL)


func _enable_fallback(reason: String) -> void:
	_use_fallback = true
	_connected = false
	_authenticated = false
	connection_state_changed.emit(false, reason)
	print("Fallback actif — ", reason)


func _process(delta: float) -> void:
	_fallback_elapsed += delta
	if _use_fallback:
		_poll_http_health()
		activity_changed.emit(ActivityMapper.fallback_idle(_fallback_elapsed))
		_reconnect_cooldown -= delta
		if _reconnect_cooldown <= 0.0:
			_reconnect_cooldown = 5.0
			_peer = WebSocketPeer.new()
			_use_fallback = false
			_connect_ws()
		return

	_peer.poll()
	var state := _peer.get_ready_state()
	match state:
		WebSocketPeer.STATE_OPEN:
			if not _connected:
				_connected = true
				_send_connect()
			while _peer.get_available_packet_count() > 0:
				_handle_packet(_peer.get_packet().get_string_from_utf8())
			_poll_timer += delta
			if _authenticated and _poll_timer >= POLL_SECS:
				_poll_timer = 0.0
				_request_health()
		WebSocketPeer.STATE_CLOSING, WebSocketPeer.STATE_CLOSED:
			if _connected:
				_enable_fallback("Connexion WS fermée")
			elif _reconnect_cooldown <= 0.0:
				_enable_fallback("Daemon non joignable")
				_reconnect_cooldown = 5.0
				_peer = WebSocketPeer.new()
				_connect_ws()
	if _reconnect_cooldown > 0.0:
		_reconnect_cooldown -= delta


func _send_connect() -> void:
	var msg := {"type": "connect", "token": _token}
	_peer.send_text(JSON.stringify(msg))


func _request_health() -> void:
	_request_id += 1
	var msg := {
		"type": "execute",
		"request_id": str(_request_id),
		"command": {"command": "HealthCheck"},
	}
	_peer.send_text(JSON.stringify(msg))


func _handle_packet(text: String) -> void:
	var data = JSON.parse_string(text)
	if typeof(data) != TYPE_DICTIONARY:
		return
	match data.get("type", ""):
		"connect_ok":
			_authenticated = true
			connection_state_changed.emit(true, "Connecté v%s" % data.get("version", "?"))
			_request_health()
		"result":
			_parse_health_result(data)
		"error":
			push_warning("Daemon: %s" % data.get("message", "erreur"))
		"pong":
			pass


func _parse_health_result(data: Dictionary) -> void:
	var response: Dictionary = data.get("response", {})
	if response.get("response") != "Health":
		return
	var payload: Dictionary = response.get("payload", {})
	var intensity := ActivityMapper.clamp_intensity(
		ActivityMapper.from_health(
			str(payload.get("status", "unknown")),
			bool(payload.get("llm_available", false)),
			bool(payload.get("embedding_available", false)),
		)
	)
	activity_changed.emit(intensity)


func _poll_http_health() -> void:
	if _http.get_http_client_status() != HTTPClient.STATUS_DISCONNECTED:
		return
	if _pending_health:
		return
	_pending_health = true
	_http.request(HEALTH_URL)


func _on_http_health(_result: int, _code: int, _headers: PackedStringArray, body: PackedByteArray) -> void:
	_pending_health = false
	if _code != 200:
		return
	var data = JSON.parse_string(body.get_string_from_utf8())
	if typeof(data) != TYPE_DICTIONARY:
		return
	# HTTP health ne donne pas llm/embedding — intensité modérée si daemon répond.
	var raw := 0.45 if data.get("status") == "ok" else 0.3
	activity_changed.emit(ActivityMapper.clamp_intensity(raw))
	connection_state_changed.emit(true, "HTTP health (fallback)")