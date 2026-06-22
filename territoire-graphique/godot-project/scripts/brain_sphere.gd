extends MeshInstance3D

@export var pulse_speed: float = 1.2
@export var base_emission: float = 1.0

var activity_intensity: float = 0.7

var websocket: WebSocketPeer

func _ready():
    websocket = WebSocketPeer.new()
    var err = websocket.connect_to_url("ws://127.0.0.1:28790")
    if err != OK:
        print("WebSocket non disponible - mode simulation")
    
    if material_override:
        material_override.set_shader_parameter("activity", activity_intensity)

func _process(delta):
    var time = Time.get_ticks_msec() / 1000.0
    
    if material_override:
        material_override.set_shader_parameter("time", time)
        material_override.set_shader_parameter("activity", activity_intensity)
    
    # Particules dynamiques
    var particles = $GPUParticles3D
    if particles:
        var target_amount = int(25 + activity_intensity * 55)
        particles.amount = target_amount
        particles.emitting = activity_intensity > 0.25

func update_brain_activity(intensity: float):
    activity_intensity = clamp(intensity, 0.0, 3.0)
    
    if material_override:
        material_override.set_shader_parameter("activity", activity_intensity)
    
    # Mise à jour du panneau si présent
    var monitoring = get_node_or_null("../MonitoringPanel")
    if monitoring and monitoring.has_method("update_activity"):
        monitoring.update_activity(activity_intensity)