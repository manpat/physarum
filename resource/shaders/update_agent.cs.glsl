layout(local_size_x=8) in;

struct Agent {
	vec2 pos;
	uint vel;
	float heading;
};

layout(binding=0, r32ui) readonly uniform uimage2D u_trail_map;

layout(binding=0) buffer AgentData {
	Agent s_agents[];
};

layout(binding=0) uniform AgentParameters {
	float u_sensor_distance;
	float u_sensor_spread;

	float u_steer_amount;

	float u_movement_speed;
	float u_inertia;
};


float read_trail(vec2 pos) {
	const ivec2 target_size = imageSize(u_trail_map);
	const ivec2 texel_coord = ivec2(floor(pos + target_size)) % target_size;
	const float raw_value = float(imageLoad(u_trail_map, texel_coord).r) / 100.0;

	return raw_value;
}


void main() {
	const uint agent_index = gl_GlobalInvocationID.x;
	if(agent_index >= s_agents.length()) {
		return;
	}

	Agent agent = s_agents[agent_index];

	const float sensor_spread = u_sensor_spread * 3.1415;
	const float steer_angle = u_steer_amount * 3.1415;

	const float heading = agent.heading;

	const vec2 forward_direction = vec2(sin(heading), cos(heading));
	const vec2 sensor_direction0 = vec2(sin(heading + sensor_spread), cos(heading + sensor_spread));
	const vec2 sensor_direction1 = vec2(sin(heading - sensor_spread), cos(heading - sensor_spread));

	const float forward_value = read_trail(agent.pos + forward_direction * u_sensor_distance);
	const float left_value = read_trail(agent.pos + sensor_direction1 * u_sensor_distance);
	const float right_value = read_trail(agent.pos + sensor_direction0 * u_sensor_distance);

	// only adjust heading if left or right value is greater
	if(forward_value < max(left_value, right_value)) {
		if(right_value > left_value) {
			agent.heading += steer_angle;
		} else {
			agent.heading -= steer_angle;
		}
	}

	const float strength = max(forward_value, max(left_value, right_value)) + 1.0;

	const vec2 desired_direction = vec2(sin(agent.heading), cos(agent.heading));
	const float desired_speed = u_movement_speed * (strength / (strength+1.0));
	const vec2 desired_vel = desired_direction * desired_speed;

	const vec2 current_vel = unpackSnorm2x16(agent.vel) * 100.0;
	const float current_speed = length(current_vel);

	vec2 velocity = current_vel + (desired_vel - current_vel) * (1.0 - u_inertia);

	agent.pos += velocity;
	agent.vel = packSnorm2x16(velocity / 100.0);

	s_agents[agent_index] = agent;
}