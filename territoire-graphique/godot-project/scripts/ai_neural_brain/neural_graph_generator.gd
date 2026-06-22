class_name NeuralGraphGenerator
extends RefCounted
## Génération procédurale : distribution sphérique + graphe k-NN synaptique.

const GOLDEN_ANGLE := PI * (3.0 - sqrt(5.0))


static func fibonacci_sphere(count: int, radius: float) -> PackedVector3Array:
	var points := PackedVector3Array()
	points.resize(count)
	if count <= 0:
		return points
	if count == 1:
		points[0] = Vector3(0.0, 0.0, radius)
		return points
	for i in range(count):
		var t := float(i) / float(count - 1)
		var y := 1.0 - t * 2.0
		var ring := sqrt(maxf(0.0, 1.0 - y * y))
		var theta := GOLDEN_ANGLE * float(i)
		points[i] = Vector3(cos(theta) * ring, y, sin(theta) * ring) * radius
	return points


static func build_knn_edges(
	positions: PackedVector3Array,
	neighbors_k: int,
	max_edge_length: float,
	seed: int,
) -> Array:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed
	var n := positions.size()
	var edge_set := {}
	var edges: Array = []

	for i in range(n):
		var distances: Array = []
		for j in range(n):
			if i == j:
				continue
			var d := positions[i].distance_to(positions[j])
			distances.append({"j": j, "d": d})
		distances.sort_custom(func(a, b): return a["d"] < b["d"])

		var added := 0
		for entry in distances:
			if added >= neighbors_k:
				break
			var j: int = entry["j"]
			var d: float = entry["d"]
			if max_edge_length > 0.0 and d > max_edge_length:
				continue
			var a := mini(i, j)
			var b := maxi(i, j)
			var key := "%d:%d" % [a, b]
			if edge_set.has(key):
				continue
			edge_set[key] = true
			edges.append(Vector2i(a, b))
			added += 1

	# Compléter le réseau si trop clairsemé (seuil relâché).
	if edges.size() < n * 2:
		var relaxed := max_edge_length * 1.35
		for i in range(n):
			var distances: Array = []
			for j in range(n):
				if i == j:
					continue
				distances.append({"j": j, "d": positions[i].distance_to(positions[j])})
			distances.sort_custom(func(a, b): return a["d"] < b["d"])
			var added := 0
			for entry in distances:
				if added >= neighbors_k + 1:
					break
				if entry["d"] > relaxed:
					break
				var j: int = entry["j"]
				var a := mini(i, j)
				var b := maxi(i, j)
				var key := "%d:%d" % [a, b]
				if edge_set.has(key):
					continue
				edge_set[key] = true
				edges.append(Vector2i(a, b))
				added += 1

	return edges


static func hash01(a: int, b: int, c: int) -> float:
	var v := sin(float(a * 127.1 + b * 311.7 + c * 74.7)) * 43758.5453
	return v - floor(v)