extends SceneTree
## Smoke test headless — arcs 3D (ruban + packet).

func _initialize() -> void:
	var lines := CommunicationLines.new()
	root.add_child(lines)

	var from := Node3D.new()
	from.position = Vector3(-2, 0.4, 0)
	var to := Node3D.new()
	to.position = Vector3(2, 0.4, 0)
	root.add_child(from)
	root.add_child(to)

	lines.register_agent("a", from)
	lines.register_agent("b", to)
	lines.on_agent_message("a", "b", "test-1")

	for _i in 30:
		lines._process(0.1)

	var arc_count: int = lines._active_arcs.size()
	if arc_count != 1:
		push_error("Expected 1 active arc, got %d" % arc_count)
		quit(1)
		return

	var arc: Dictionary = lines._active_arcs[0]
	var ribbon: MeshInstance3D = arc.get("ribbon")
	if ribbon == null or ribbon.mesh == null:
		push_error("Ribbon mesh not built")
		quit(1)
		return
	if arc.get("packet") == null or arc.get("halo") == null:
		push_error("Packet/halo missing")
		quit(1)
		return
	var trails: Array = arc.get("trails", [])
	if trails.size() != 5:
		push_error("Expected 5 trail sparks")
		quit(1)
		return
	if arc.get("burst_from") == null:
		push_error("Burst ring missing")
		quit(1)
		return

	print("communication_lines 3D smoke test OK")
	quit(0)