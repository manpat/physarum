layout(local_size_x=8, local_size_y=8) in;

layout(binding=0, r32ui) readonly uniform uimage2D u_trail_map;
layout(binding=1, rgba8) uniform image2D u_draw_buffer;

void main() {
	ivec2 write_coord = ivec2(gl_GlobalInvocationID.xy);

	ivec2 target_size = imageSize(u_draw_buffer);
	if (any(greaterThanEqual(write_coord, target_size))) {
		return;
	}

	ivec2 trail_coord = (write_coord + target_size / 2) % target_size;

	float trail_value = float(imageLoad(u_trail_map, trail_coord).r) / 100.0;

	vec3 color = vec3(0.0);

	const float r_factor = 0.5;
	const float g_factor = 5.0;
	const float b_factor = 20.0;

	color.r = trail_value / r_factor;
	color.g = max(trail_value - r_factor, 0.0) / g_factor;
	color.b = max(trail_value - g_factor, 0.0) / b_factor;

	color = color / (color + vec3(1.0));

	imageStore(u_draw_buffer, write_coord, vec4(color, 1.0));
}