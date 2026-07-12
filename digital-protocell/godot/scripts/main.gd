extends Control

@onready var sim: Node = $ChemistrySimulator
@onready var display: TextureRect = $Display
@onready var panel: Control = $DiagnosticsPanel

var _image: Image
var _texture: ImageTexture
var _width: int = 192
var _height: int = 192
var _paused: bool = false

func _ready() -> void:
	if sim.has_method("get_grid_width"):
		_width = int(sim.get_grid_width())
		_height = int(sim.get_grid_height())
	_image = Image.create(_width, _height, false, Image.FORMAT_RGB8)
	_texture = ImageTexture.create_from_image(_image)
	display.texture = _texture
	if display.material:
		display.material.set_shader_parameter("chemistry_texture", _texture)

func _process(_delta: float) -> void:
	_update_texture()
	_update_diagnostics()

func _update_texture() -> void:
	for y in range(_height):
		for x in range(_width):
			var phi: float = sim.get_structure_at(x, y) if sim.has_method("get_structure_at") else 0.0
			var cat: float = sim.get_catalyst_at(x, y) if sim.has_method("get_catalyst_at") else 0.0
			var wst: float = sim.get_waste_at(x, y) if sim.has_method("get_waste_at") else 0.0
			var r := int(clamp(phi * 255.0, 0.0, 255.0))
			var g := int(clamp(cat * 255.0, 0.0, 255.0))
			var b := int(clamp(wst * 255.0, 0.0, 255.0))
			_image.set_pixel(x, y, Color8(r, g, b))
	_texture.update(_image)

func _update_diagnostics() -> void:
	if panel and panel.has_method("set_diagnostics"):
		panel.set_diagnostics({
			"time": sim.get_sim_time() if sim.has_method("get_sim_time") else 0.0,
			"dt": sim.get_dt() if sim.has_method("get_dt") else 0.0,
			"structure_mass": sim.get_structural_mass() if sim.has_method("get_structural_mass") else 0.0,
			"catalyst_mass": sim.get_catalyst_mass() if sim.has_method("get_catalyst_mass") else 0.0,
			"classification": sim.get_classification() if sim.has_method("get_classification") else "UNKNOWN",
		})

func _on_pause_pressed() -> void:
	_paused = not _paused
	if _paused:
		sim.pause_sim()
	else:
		sim.resume_sim()

func _on_step_pressed() -> void:
	sim.single_substep()

func _on_reset_pressed() -> void:
	sim.reset_experiment()

func _on_puncture_pressed() -> void:
	sim.run_puncture()

func _on_remove_nutrient_pressed() -> void:
	sim.remove_nutrient()

func _on_remove_fuel_pressed() -> void:
	sim.remove_fuel()

func _on_disable_rep_pressed() -> void:
	sim.disable_catalyst_reproduction()

func _on_restore_pressed() -> void:
	sim.restore_reservoir()

func _on_save_pressed() -> void:
	var path := "user://snapshot.json"
	sim.save_snapshot(path)
