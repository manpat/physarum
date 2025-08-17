use toybox::prelude::*;

fn main() -> anyhow::Result<()> {
	toybox::run("physarum", App::new)
}

struct App {
	agent_buffer: gfx::BufferName,
	num_agents: i32,

	trail_buffer: gfx::ImageHandle,
	old_trail_buffer: gfx::ImageHandle,
	draw_buffer: gfx::ImageHandle,

	update_agent_cs: gfx::ShaderHandle,
	write_trail_cs: gfx::ShaderHandle,
	blur_trail_cs: gfx::ShaderHandle,
	fade_trail_cs: gfx::ShaderHandle,
	update_draw_buffer_cs: gfx::ShaderHandle,

	agent_parameters: AgentParameters,
	blur_passes: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct AgentState {
	pos: Vec2,
	vel: [i16; 2],
	heading: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct AgentParameters {
	sensor_distance: f32,
	sensor_spread: f32,

	steer_amount: f32,

	movement_speed: f32,
	inertia: f32,

}

fn gen_starting_conditions(size: Vec2) -> Vec<AgentState> {
	use rand::random_range;

	let mut v = Vec::new();

	for _ in 0..5_000_000 {
		v.push(AgentState {
			pos: Vec2::new(random_range(0.0 ..= size.x), random_range(0.0 ..= size.y)),
			vel: [random_range(-256..=256), random_range(-256..=256)],
			heading: random_range(0.0 ..= TAU),
		});
	}

	v
}

impl App {
	fn new(ctx: &mut toybox::Context) -> anyhow::Result<App> {
		let gfx::System{ core, resource_manager, frame_encoder, .. } = &mut *ctx.gfx;

		let num_agents = 5_000_000;

		let agent_buffer = core.create_buffer();
		core.allocate_buffer_storage(agent_buffer, num_agents * size_of::<AgentState>(), 0);

		let num_agents = num_agents as i32;

		let trail_buffer_request = gfx::CreateImageRequest::rendertarget("trail0", gfx::ImageFormat::Red(gfx::ComponentFormat::U32))
			.clear_policy(gfx::ImageClearPolicy::Never)
			.resize_to_backbuffer_fraction(1);
		let trail_buffer_request2 = gfx::CreateImageRequest::rendertarget("trail1", gfx::ImageFormat::Red(gfx::ComponentFormat::U32))
			.clear_policy(gfx::ImageClearPolicy::Never)
			.resize_to_backbuffer_fraction(1);

		let draw_buffer_request = gfx::CreateImageRequest::rendertarget("draw", gfx::ImageFormat::Srgba8)
			.clear_policy(gfx::ImageClearPolicy::Never)
			.resize_to_backbuffer_fraction(1);

		let trail_buffer = resource_manager.request(trail_buffer_request);
		let old_trail_buffer = resource_manager.request(trail_buffer_request2);
		let draw_buffer = resource_manager.request(draw_buffer_request);

		// Init agent buffer
		{
			let init_agent_cs = resource_manager.load_compute_shader("shaders/init_agents.cs.glsl");

			let mut group = frame_encoder.command_group(gfx::FrameStage::Start);
			group.compute(init_agent_cs)
				.groups(((num_agents + 7)/8, 1, 1))
				.ssbo(0, agent_buffer)
				.image(0, trail_buffer);
		}

		Ok(App {
			update_agent_cs: resource_manager.load_compute_shader("shaders/update_agent.cs.glsl"),
			update_draw_buffer_cs: resource_manager.load_compute_shader("shaders/update_draw_buffer.cs.glsl"),
			write_trail_cs: resource_manager.load_compute_shader("shaders/write_trail.cs.glsl"),
			blur_trail_cs: resource_manager.load_compute_shader("shaders/blur_trail.cs.glsl"),
			fade_trail_cs: resource_manager.load_compute_shader("shaders/fade_trail.cs.glsl"),

			trail_buffer,
			old_trail_buffer,
			draw_buffer,

			agent_buffer,
			num_agents,

			agent_parameters: AgentParameters {
				sensor_distance: 10.0,
				sensor_spread: 0.2,

				steer_amount: 0.2,

				movement_speed: 5.0,
				inertia: 0.0,
			},

			blur_passes: 2,
		})
	}
}

impl toybox::App for App {
	fn present(&mut self, ctx: &mut toybox::Context) {
		egui::Window::new("Parameters")
			.show(&ctx.egui, |ui| {
				let params = &mut self.agent_parameters;

				ui.add(egui::Slider::new(&mut params.sensor_distance, 0.0..=50.0).text("Sensor Distance"));
				ui.add(egui::Slider::new(&mut params.sensor_spread, 0.0..=1.0).text("Sensor Spread"));

				ui.add(egui::Slider::new(&mut params.steer_amount, 0.0..=1.0).text("Steer Amount"));

				ui.add(egui::Slider::new(&mut params.movement_speed, 0.0..=50.0).text("Movement Speed"));
				ui.add(egui::Slider::new(&mut params.inertia, 0.0..=1.0).text("Inertia"));

				ui.add(egui::Slider::new(&mut self.blur_passes, 0..=10).text("Blur Passes"));
			});

		let mut group = ctx.gfx.frame_encoder.command_group(gfx::FrameStage::Main);

		group.bind_shared_ubo(0, &[self.agent_parameters.clone()]);

		for _ in 0..2 {
			for _ in 0..self.blur_passes {
				group.compute(self.blur_trail_cs)
					.groups_from_image_size(self.trail_buffer)
					.image(0, self.trail_buffer)
					.image_rw(1, self.old_trail_buffer);

				std::mem::swap(&mut self.old_trail_buffer, &mut self.trail_buffer);
			}

			group.compute(self.fade_trail_cs)
				.groups_from_image_size(self.trail_buffer)
				.image(0, self.trail_buffer)
				.image_rw(1, self.old_trail_buffer);

			std::mem::swap(&mut self.old_trail_buffer, &mut self.trail_buffer);

			group.compute(self.update_agent_cs)
				.groups(((self.num_agents + 7)/8, 1, 1))
				.ssbo(0, self.agent_buffer)
				.image(0, self.trail_buffer);

			group.compute(self.write_trail_cs)
				.groups(((self.num_agents + 7)/8, 1, 1))
				.ssbo(0, self.agent_buffer)
				.image_rw(0, self.trail_buffer);
		}


		let mut group = ctx.gfx.frame_encoder.command_group(gfx::FrameStage::Postprocess);

		group.compute(self.update_draw_buffer_cs)
			.groups_from_image_size(self.draw_buffer)
			.image(0, self.trail_buffer)
			.image_rw(1, self.draw_buffer);

		group.draw_fullscreen(None)
			.sampled_image(0, self.draw_buffer, gfx::CommonSampler::LinearRepeat);
	}
}