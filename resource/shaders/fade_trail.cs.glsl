layout(local_size_x=8, local_size_y=8) in;

layout(binding=0, r32ui) readonly uniform uimage2D u_read_trail_map;
layout(binding=1, r32ui) writeonly uniform uimage2D u_write_trail_map;

uint read_trail(ivec2 texel_coord) {
	ivec2 target_size = imageSize(u_read_trail_map);
	texel_coord = (texel_coord + target_size) % target_size;
	return imageLoad(u_read_trail_map, texel_coord).r;
}

void main() {
	ivec2 texel_coord = ivec2(gl_GlobalInvocationID.xy);

	ivec2 target_size = imageSize(u_read_trail_map);
	if (any(greaterThanEqual(texel_coord, target_size))) {
		return;
	}

	uint value = read_trail(texel_coord);

	// value -= value/5000*500;
	// value -= value/500*100;
	// value -= value/300*50;
	value -= value/4;

	if(value > 0) {
		value -= 1;
	}

	imageStore(u_write_trail_map, texel_coord, uvec4(value, 0, 0, 1));
}