shader_type canvas_item;

uniform sampler2D chemistry_texture : filter_linear;
uniform float catalyst_glow : hint_range(0.0, 1.0) = 0.3;
uniform float waste_tint : hint_range(0.0, 1.0) = 0.2;

void fragment() {
	vec4 base = texture(chemistry_texture, UV);
	float structure = base.r;
	float catalyst = base.g;
	float waste = base.b;
	vec3 color = vec3(structure * 0.2, structure * 0.6 + catalyst * catalyst_glow, structure * 0.3 + waste * waste_tint);
	COLOR = vec4(color, smoothstep(0.02, 0.15, structure + catalyst * 0.5));
}
