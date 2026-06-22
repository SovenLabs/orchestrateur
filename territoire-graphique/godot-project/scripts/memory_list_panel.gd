class_name MemoryListPanel
extends DockPanel
## Panneau Memory List — liste, recherche, détail.

signal memory_selected(memory_id: String)

@onready var _search: LineEdit = %SearchField
@onready var _list: ItemList = %MemoryList
@onready var _detail: RichTextLabel = %MemoryDetail
@onready var _refresh_btn: Button = %RefreshButton

var _daemon: TerritoryDaemonClient
var _items: Array = []
var _pending_list_rid := ""
var _pending_detail_rid := ""
var _id_by_row: Dictionary = {}


func _ready() -> void:
	panel_id = "memory"
	panel_title = "Mémoires"
	super._ready()
	_daemon = TerritoryDaemonClient.resolve(self)
	_list.item_selected.connect(_on_item_selected)
	_refresh_btn.pressed.connect(_refresh_list)
	_search.text_submitted.connect(_on_search)
	if _daemon:
		_daemon.command_completed.connect(_on_command_completed)
		_daemon.connection_state_changed.connect(func(c, _d): if c: _refresh_list())
		_daemon.broadcast_received.connect(_on_broadcast)


func _refresh_list() -> void:
	if not _daemon:
		return
	_pending_list_rid = _daemon.execute_list(_search.text.strip_edges())


func _on_search(_text: String) -> void:
	if not _daemon:
		return
	var q := _search.text.strip_edges()
	if q.is_empty():
		_refresh_list()
	else:
		_pending_list_rid = _daemon.execute_search(q)


func focus_memory(memory_id: String) -> void:
	for idx in _id_by_row:
		if _id_by_row[idx] == memory_id:
			_list.select(idx)
			_on_item_selected(idx)
			return
	if _daemon:
		_pending_detail_rid = _daemon.execute_get_memory(memory_id)
	memory_selected.emit(memory_id)


func _on_item_selected(index: int) -> void:
	if not _id_by_row.has(index):
		return
	var mem_id: String = _id_by_row[index]
	memory_selected.emit(mem_id)
	if _daemon:
		_pending_detail_rid = _daemon.execute_get_memory(mem_id)


func _on_broadcast(event: String, _payload: Dictionary, _source: String) -> void:
	if event == "memories_changed":
		_refresh_list()


func _on_command_completed(request_id: String, response: Dictionary) -> void:
	var kind: String = str(response.get("response", ""))
	var payload: Dictionary = response.get("payload", {})

	if request_id == _pending_list_rid:
		_pending_list_rid = ""
		_populate_list(kind, payload)
	elif request_id == _pending_detail_rid:
		_pending_detail_rid = ""
		_show_detail(kind, payload)


func _populate_list(kind: String, payload: Dictionary) -> void:
	_list.clear()
	_id_by_row.clear()
	_items.clear()

	var rows: Array = []
	if kind == "MemoryList":
		rows = payload.get("items", [])
	elif kind == "SearchResults":
		rows = payload.get("items", [])

	for i in range(rows.size()):
		var row: Dictionary = rows[i]
		var title: String = str(row.get("title", "Sans titre"))
		var mem_id: String = str(row.get("id", row.get("memory_id", "")))
		if kind == "SearchResults":
			var score = row.get("score", 0.0)
			title = "%s (%.2f)" % [title, float(score)]
		var idx := _list.add_item(title)
		_id_by_row[idx] = mem_id
		_items.append(row)


func _show_detail(kind: String, payload: Dictionary) -> void:
	if kind != "MemoryDetail":
		_detail.text = "Détail indisponible."
		return
	var memory: Dictionary = payload.get("memory", {})
	var title: String = str(memory.get("title", ""))
	var content: String = str(memory.get("content", ""))
	var tags: Array = memory.get("tags", [])
	_detail.text = "[b]%s[/b]\nTags: %s\n\n%s" % [title, ", ".join(tags), content]