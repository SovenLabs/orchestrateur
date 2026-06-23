extends Node3D
class_name CommunicationLines
## Arcs 3D synapse-style entre agents — ruban courbe, pulse, traînée, éclats (Phase 5).

const ARC_DURATION := 3.0
const ARC_SEGMENTS := 52
const ARC_WIDTH := 0.09
const ARC_LIFT_FACTOR := 0.42
const MAX_ACTIVE_ARCS := 20
const TRAIL_COUNT := 5
const TRAIL_SPACING := 0.045
const BURST_DURATION := 0.55

const ARC_SHADER := preload("res://shaders/agents/communication_arc.gdshader")
const PACKET_SHADER := preload("res://shaders/agents/communication_packet.gdshader")
const BURST_SHADER := preload("res://shaders/agents/communication_burst.gdshader")

var _agent_nodes: Dictionary = {}
var _active_arcs: Array = []
var _ribbon_pool: Array[MeshInstance3D] = []
var _packet_pool: Array[MeshInstance3D] = []
var _halo_pool: Array[MeshInstance3D] = []
var _trail_pool: Array[MeshInstance3D] = []
var _burst_pool: Array[MeshInstance3D] = []
var _time := 0.0
var _arc_counter := 0


func _ready() -> void:
	set_process(true)


func register_agent(agent_id: String, node: Node3D) -> void:
	_agent_nodes[agent_id] = node


func unregister_agent(agent_id: String) -> void:
	_agent_nodes.erase(agent_id)
	for i in range(_active_arcs.size() - 1, -1, -1):
		var arc: Dictionary = _active_arcs[i]
		if arc.get("from_id") == agent_id or arc.get("to_id") == agent_id:
			_recycle_arc(arc)
			_active_arcs.remove_at(i)


func on_agent_message(from_id: String, to_id: String, _message_id: String) -> void:
	var from_node: Node3D = _agent_nodes.get(from_id)
	var to_node: Node3D = _agent_nodes.get(to_id)
	if from_node == null or to_node == null:
		return

	while _active_arcs.size() >= MAX_ACTIVE_ARCS:
		_recycle_arc(_active_arcs.pop_front())

	_arc_counter += 1
	var seed := float(_arc_counter % 97) + from_id.hash() * 0.001

	var arc := {
		"from_id": from_id,
		"to_id": to_id,
		"ribbon": _acquire_ribbon(seed),
		"packet": _acquire_packet(),
		"halo": _acquire_halo(),
		"trails": _acquire_trails(),
		"burst_from": _spawn_burst(from_node.global_position, seed),
		"burst_to": null,
		"age": 0.0,
		"duration": ARC_DURATION,
		"seed": seed,
		"arrival_done": false,
	}
	_active_arcs.append(arc)

	if from_node.has_method("pulse_on_message"):
		from_node.pulse_on_message()
	if to_node.has_method("pulse_on_message"):
		to_node.pulse_on_message()


func _process(delta: float) -> void:
	_time += delta
	for i in range(_active_arcs.size() - 1, -1, -1):
		var arc: Dictionary = _active_arcs[i]
		arc["age"] = float(arc.get("age", 0.0)) + delta
		var age: float = arc["age"]
		var duration: float = float(arc.get("duration", ARC_DURATION))
		if age >= duration:
			_recycle_arc(arc)
			_active_arcs.remove_at(i)
			continue
		_update_arc_visual(arc, age, duration)


func _update_arc_visual(arc: Dictionary, age: float, duration: float) -> void:
	var from_node: Node3D = _agent_nodes.get(str(arc.get("from_id", "")))
	var to_node: Node3D = _agent_nodes.get(str(arc.get("to_id", "")))
	if from_node == null or to_node == null:
		return

	var from_l: Vector3 = to_local(from_node.global_position)
	var to_l: Vector3 = to_local(to_node.global_position)
	var seed: float = float(arc.get("seed", 0.0))
	var c1: Vector3
	var c2: Vector3
	_arc_controls(from_l, to_l, seed, c1, c2)

	var life: float = 1.0 - age / duration
	var pulse_t: float = clampf(age / duration, 0.0, 1.0)
	var fade_in: float = smoothstep(0.0, 0.08, pulse_t)
	var alpha: float = life * fade_in
	var arrival_flash := 0.0

	if pulse_t >= 0.92 and not arc.get("arrival_done", false):
		arc["arrival_done"] = true
		arc["burst_to_start"] = age
		arc["burst_to"] = _spawn_burst(to_node.global_position, seed + 17.0)
	if pulse_t >= 0.92:
		arrival_flash = smoothstep(0.92, 0.98, pulse_t) * life

	var ribbon: MeshInstance3D = arc.get("ribbon")
	if ribbon:
		ribbon.mesh = _build_ribbon_mesh(from_l, c1, c2, to_l)
		if ribbon.material_override is ShaderMaterial:
			var mat := ribbon.material_override as ShaderMaterial
			mat.set_shader_parameter("time", _time)
			mat.set_shader_parameter("pulse_pos", pulse_t)
			mat.set_shader_parameter("alpha_mul", alpha)
			mat.set_shader_parameter("arc_seed", seed)
			mat.set_shader_parameter("arrival_flash", arrival_flash)

	var packet: MeshInstance3D = arc.get("packet")
	var halo: MeshInstance3D = arc.get("halo")
	if packet:
		var pos := _cubic_bezier(from_l, c1, c2, to_l, pulse_t)
		var tangent := _cubic_bezier_tangent(from_l, c1, c2, to_l, pulse_t).normalized()
		packet.position = pos
		packet.scale = Vector3.ONE * lerpf(0.1, 0.22, sin(pulse_t * PI))
		if packet.material_override is ShaderMaterial:
			var pmat := packet.material_override as ShaderMaterial
			pmat.set_shader_parameter("time", _time)
			pmat.set_shader_parameter("alpha_mul", alpha)
			pmat.set_shader_parameter("tail_stretch", lerpf(0.4, 1.2, pulse_t))
		if halo:
			halo.position = pos
			halo.scale = Vector3.ONE * lerpf(0.35, 0.65, sin(pulse_t * PI * 2.0))
			if halo.material_override is ShaderMaterial:
				var hmat := halo.material_override as ShaderMaterial
				hmat.set_shader_parameter("time", _time)
				hmat.set_shader_parameter("alpha_mul", alpha * 0.45)
				hmat.set_shader_parameter("tail_stretch", 0.2)

	var trails: Array = arc.get("trails", [])
	for ti in trails.size():
		var trail: MeshInstance3D = trails[ti]
		if trail == null:
			continue
		var lag := float(ti + 1) * TRAIL_SPACING
		var tt := clampf(pulse_t - lag, 0.0, 1.0)
		trail.position = _cubic_bezier(from_l, c1, c2, to_l, tt)
		var trail_alpha := alpha * (1.0 - float(ti + 1) / float(TRAIL_COUNT + 1))
		trail.scale = Vector3.ONE * lerpf(0.03, 0.07, 1.0 - float(ti) / float(TRAIL_COUNT))
		trail.visible = pulse_t > lag
		if trail.material_override is StandardMaterial3D:
			var tmat := trail.material_override as StandardMaterial3D
			tmat.albedo_color.a = trail_alpha

	_update_burst(arc.get("burst_from"), age, alpha)
	var burst_to_start: float = float(arc.get("burst_to_start", -1.0))
	if burst_to_start >= 0.0:
		_update_burst(arc.get("burst_to"), age - burst_to_start, alpha)


func _update_burst(burst: Variant, local_age: float, parent_alpha: float) -> void:
	if burst == null or not burst is MeshInstance3D:
		return
	var mi: MeshInstance3D = burst
	if local_age < 0.0:
		mi.visible = false
		return
	var progress := clampf(local_age / BURST_DURATION, 0.0, 1.0)
	if progress >= 1.0:
		mi.visible = false
		return
	mi.visible = true
	mi.scale = Vector3.ONE * lerpf(0.3, 1.8, progress)
	if mi.material_override is ShaderMaterial:
		var mat := mi.material_override as ShaderMaterial
		mat.set_shader_parameter("ring_progress", progress)
		mat.set_shader_parameter("alpha_mul", parent_alpha * (1.0 - progress))


func _build_ribbon_mesh(p0: Vector3, p1: Vector3, p2: Vector3, p3: Vector3) -> ArrayMesh:
	var tangent_start := _cubic_bezier_tangent(p0, p1, p2, p3, 0.0).normalized()
	var tangent_end := _cubic_bezier_tangent(p0, p1, p2, p3, 1.0).normalized()
	var prev_right := _safe_right(tangent_start)

	var vertices := PackedVector3Array()
	var uvs := PackedVector2Array()
	var normals := PackedVector3Array()
	var indices := PackedInt32Array()

	for seg in range(ARC_SEGMENTS + 1):
		var t := float(seg) / float(ARC_SEGMENTS)
		var center := _cubic_bezier(p0, p1, p2, p3, t)
		var tangent := _cubic_bezier_tangent(p0, p1, p2, p3, t)
		if tangent.length_squared() < 0.0001:
			tangent = tangent_end if t > 0.5 else tangent_start
		tangent = tangent.normalized()
		var right := _safe_right(tangent)
		if seg > 0:
			right = prev_right.lerp(right, 0.5).normalized()
		prev_right = right

		var envelope := sin(t * PI)
		var half_w := ARC_WIDTH * (0.5 + 0.5 * envelope)
		vertices.append(center - right * half_w)
		vertices.append(center + right * half_w)
		uvs.append(Vector2(t, 0.0))
		uvs.append(Vector2(t, 1.0))
		var normal := right.cross(tangent).normalized()
		normals.append(normal)
		normals.append(normal)

		if seg > 0:
			var base := (seg - 1) * 2
			indices.append_array([base, base + 1, base + 2, base + 1, base + 3, base + 2])

	var arrays := []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = vertices
	arrays[Mesh.ARRAY_TEX_UV] = uvs
	arrays[Mesh.ARRAY_NORMAL] = normals
	arrays[Mesh.ARRAY_INDEX] = indices
	var mesh := ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	return mesh


func _arc_controls(from_p: Vector3, to_p: Vector3, seed: float, out_c1: Vector3, out_c2: Vector3) -> void:
	var dir := (to_p - from_p).normalized()
	var perp := _safe_right(dir)
	var dist := from_p.distance_to(to_p)
	var lift := dist * ARC_LIFT_FACTOR
	var sway := dist * 0.1 * sin(seed * 1.7)
	out_c1 = from_p.lerp(to_p, 0.28) + Vector3.UP * lift * 0.85 + perp * sway
	out_c2 = from_p.lerp(to_p, 0.72) + Vector3.UP * lift * 1.1 - perp * sway * 0.7


func _cubic_bezier(a: Vector3, b: Vector3, c: Vector3, d: Vector3, t: float) -> Vector3:
	var u := 1.0 - t
	return (
		u * u * u * a
		+ 3.0 * u * u * t * b
		+ 3.0 * u * t * t * c
		+ t * t * t * d
	)


func _cubic_bezier_tangent(a: Vector3, b: Vector3, c: Vector3, d: Vector3, t: float) -> Vector3:
	var u := 1.0 - t
	return (
		3.0 * u * u * (b - a)
		+ 6.0 * u * t * (c - b)
		+ 3.0 * t * t * (d - c)
	)


func _safe_right(tangent: Vector3) -> Vector3:
	var up := Vector3.UP
	var right := up.cross(tangent)
	if right.length_squared() < 0.0001:
		right = Vector3.RIGHT.cross(tangent)
	return right.normalized()


func _acquire_ribbon(seed: float) -> MeshInstance3D:
	var mi: MeshInstance3D
	if _ribbon_pool.is_empty():
		mi = MeshInstance3D.new()
		mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		var mat := ShaderMaterial.new()
		mat.shader = ARC_SHADER
		mi.material_override = mat
		add_child(mi)
	else:
		mi = _ribbon_pool.pop_back()
	if mi.material_override is ShaderMaterial:
		(mi.material_override as ShaderMaterial).set_shader_parameter("arc_seed", seed)
	mi.visible = true
	return mi


func _acquire_packet() -> MeshInstance3D:
	var mi: MeshInstance3D
	if _packet_pool.is_empty():
		mi = MeshInstance3D.new()
		mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		var sphere := SphereMesh.new()
		sphere.radius = 0.1
		sphere.height = 0.2
		mi.mesh = sphere
		var mat := ShaderMaterial.new()
		mat.shader = PACKET_SHADER
		mi.material_override = mat
		add_child(mi)
	else:
		mi = _packet_pool.pop_back()
	mi.visible = true
	return mi


func _acquire_halo() -> MeshInstance3D:
	var mi: MeshInstance3D
	if _halo_pool.is_empty():
		mi = MeshInstance3D.new()
		mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		var sphere := SphereMesh.new()
		sphere.radius = 0.18
		sphere.height = 0.36
		mi.mesh = sphere
		var mat := ShaderMaterial.new()
		mat.shader = PACKET_SHADER
		mat.set_shader_parameter("core_color", Vector3(0.2, 0.75, 1.0))
		mat.set_shader_parameter("halo_color", Vector3(0.55, 0.35, 1.0))
		mi.material_override = mat
		add_child(mi)
	else:
		mi = _halo_pool.pop_back()
	mi.visible = true
	return mi


func _acquire_trails() -> Array:
	var trails: Array = []
	for _i in TRAIL_COUNT:
		var mi: MeshInstance3D
		if _trail_pool.is_empty():
			mi = MeshInstance3D.new()
			mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
			var sphere := SphereMesh.new()
			sphere.radius = 0.05
			sphere.height = 0.1
			mi.mesh = sphere
			var mat := StandardMaterial3D.new()
			mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
			mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
			mat.blend_mode = BaseMaterial3D.BLEND_MODE_ADD
			mat.albedo_color = Color(0.55, 0.85, 1.0, 0.6)
			mat.emission_enabled = true
			mat.emission = Color(0.4, 0.75, 1.0)
			mat.emission_energy_multiplier = 1.8
			mi.material_override = mat
			add_child(mi)
		else:
			mi = _trail_pool.pop_back()
		mi.visible = false
		trails.append(mi)
	return trails


func _spawn_burst(global_pos: Vector3, seed: float) -> MeshInstance3D:
	var mi: MeshInstance3D
	if _burst_pool.is_empty():
		mi = MeshInstance3D.new()
		mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		var quad := QuadMesh.new()
		quad.size = Vector2(1.2, 1.2)
		mi.mesh = quad
		var mat := ShaderMaterial.new()
		mat.shader = BURST_SHADER
		mi.material_override = mat
		add_child(mi)
	else:
		mi = _burst_pool.pop_back()
	mi.position = to_local(global_pos) + Vector3(0.0, 0.05, 0.0)
	mi.rotation_degrees = Vector3(-90.0, 0.0, 0.0)
	mi.scale = Vector3.ONE * 0.3
	mi.visible = true
	if mi.material_override is ShaderMaterial:
		var bmat := mi.material_override as ShaderMaterial
		var hue_shift := sin(seed) * 0.5 + 0.5
		var col := Color(0.0, 0.87, 1.0).lerp(Color(0.73, 0.53, 1.0), hue_shift)
		bmat.set_shader_parameter("burst_color", Vector3(col.r, col.g, col.b))
		bmat.set_shader_parameter("ring_progress", 0.0)
		bmat.set_shader_parameter("alpha_mul", 1.0)
	return mi


func _recycle_arc(arc: Dictionary) -> void:
	var ribbon: MeshInstance3D = arc.get("ribbon")
	if ribbon:
		ribbon.visible = false
		_ribbon_pool.append(ribbon)
	var packet: MeshInstance3D = arc.get("packet")
	if packet:
		packet.visible = false
		_packet_pool.append(packet)
	var halo: MeshInstance3D = arc.get("halo")
	if halo:
		halo.visible = false
		_halo_pool.append(halo)
	for trail in arc.get("trails", []):
		if trail:
			trail.visible = false
			_trail_pool.append(trail)
	for key in ["burst_from", "burst_to"]:
		var burst: MeshInstance3D = arc.get(key)
		if burst:
			burst.visible = false
			_burst_pool.append(burst)