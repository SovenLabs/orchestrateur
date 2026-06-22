extends MeshInstance3D

@export var pulse_speed: float = 1.0
@export var base_emission: float = 0.9

var activity_intensity: float = 0.6

var websocket: WebSocketPeer

func _ready():
    # Initialisation WebSocket (Option B)
    websocket = WebSocketPeer.new()
    var err = websocket.connect_to_url("ws://127.0.0.1:28790")
    if err != OK:
        print("WebSocket connection failed, falling back to GDExtension simulation")
    
    # Connecter au shader
    if material_override:
        material_override.set_shader_parameter("activity", activity_intensity)

func _process(delta):
    var time = Time.get_ticks_msec() / 1000.0
    
    if material_override:
        material_override.set_shader_parameter("time", time)
        material_override.set_shader_parameter("activity", activity_intensity)
    
    # Contrôle des particules selon l'activité
    var particles = $GPUParticles3D
    if particles:
        particles.amount = int(30 + activity_intensity * 40)
        particles.emitting = activity_intensity > 0.3

# Appelé depuis le backend (via WebSocket ou GDExtension)
func update_brain_activity(intensity: float):
    activity_intensity = clamp(intensity, 0.0, 2.5)
    
    # Mise à jour immédiate du shader
    if material_override:
        material_override.set_shader_parameter("activity", activity_intensity)