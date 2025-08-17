layout(local_size_x=8) in;

struct Agent {
	vec2 pos;
	uint vel;
	float heading;
};

layout(binding=0, r32ui) uniform uimage2D u_trail_map;

layout(binding=0) buffer AgentData {
	Agent s_agents[];
};

void main() {
	const uint agent_index = gl_GlobalInvocationID.x;
	if(agent_index >= s_agents.length()) {
		return;
	}

	const ivec2 trail_map_size = imageSize(u_trail_map);

	Agent agent = s_agents[agent_index];

	// Make sure we stay within the bounds of the image
	agent.pos = mod(agent.pos + vec2(trail_map_size), vec2(trail_map_size));
	agent.pos = mod(agent.pos + vec2(trail_map_size), vec2(trail_map_size));

	s_agents[agent_index].pos = agent.pos;

	ivec2 texel_coord = ivec2(agent.pos);
	imageAtomicAdd(u_trail_map, texel_coord, 100);
}