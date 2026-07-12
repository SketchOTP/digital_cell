extends Control

@onready var label: RichTextLabel = $RichTextLabel

func set_diagnostics(d: Dictionary) -> void:
	label.text = """[b]Digital Protocell — Phase 1[/b]
Time: %.4f  dt: %.6f
Structural mass: %.1f
Catalyst mass: %.3f
Classification: %s
""" % [
		d.get("time", 0.0),
		d.get("dt", 0.0),
		d.get("structure_mass", 0.0),
		d.get("catalyst_mass", 0.0),
		str(d.get("classification", "?")),
	]
