extends Control

@onready var activity_bar = $Panel/VBoxContainer/ActivityBar
@onready var intensity_label = $Panel/VBoxContainer/IntensityLabel

var current_intensity: float = 0.6

func update_activity(intensity: float):
    current_intensity = intensity
    activity_bar.value = intensity
    intensity_label.text = "Intensity: %.2f" % intensity

func _ready():
    # Simulation initiale
    update_activity(0.6)