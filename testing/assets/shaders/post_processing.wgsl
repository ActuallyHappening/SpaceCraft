// This shader computes the chromatic aberration effect

// Since post processing is a fullscreen effect, we use the fullscreen vertex shader provided by bevy.
// This will import a vertex shader that renders a single fullscreen triangle.
//
// A fullscreen triangle is a single triangle that covers the entire screen.
// The box in the top left in that diagram is the screen. The 4 x are the corner of the screen
//
// Y axis
//  1 |  x-----x......
//  0 |  |  s  |  . ´
// -1 |  x_____x´
// -2 |  :  .´
// -3 |  :´
//    +---------------  X axis
//      -1  0  1  2  3
//
// As you can see, the triangle ends up bigger than the screen.
//
// You don't need to worry about this too much since bevy will compute the correct UVs for you.
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct PostProcessSettings {
    intensity: f32,
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

@fragment
fn fragment_entry(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Chromatic aberration strength
    let offset_strength = settings.intensity * 10.0;

    // Sample each color channel with an arbitrary shift
    var original_point = vec4<f32>(
        // textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(offset_strength, -offset_strength)).r,
        // textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(-offset_strength, 0.0)).g,
        // textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(0.0, offset_strength)).b,
				// 0.0,
				// 0.0,
				textureSample(screen_texture, texture_sampler, in.uv).r,
				textureSample(screen_texture, texture_sampler, in.uv).g,
				textureSample(screen_texture, texture_sampler, in.uv).b,
        1.0
    );

		// let _uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
		// let uv = _uv * 2.0 - 1.0;

		// let d = length(uv);

		// let original_point = vec4<f32>(
		// 	// 0.0,
		// 	uv.x,
		// 	// in.position.x,
		// 	uv.y,
		// 	// in.position.y,
		// 	// 0.0,
		// 	// in.position.z,
		// 	// in.uv.z
		// 	0.0,
		// 	1.0
		// );


		// // if original_point.r == 0.0 && original_point.g == 0.0 && original_point.b == 0.0 {
		// if original_point.a == 0.0 {
		// 	original_point = vec4<f32>(0.0, 1.0, 0.0, 1.0);
		// } else {
		// 	// original_point = vec4<f32>(0.0, 1.0, 0.0, 1.0);
		// }

		return original_point;

    // return vec4<f32>(
    //     textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(offset_strength, -offset_strength)).r,
    //     textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(-offset_strength, 0.0)).g,
    //     textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(0.0, offset_strength)).b,
    //     1.0
    // );
		// return vec4<f32>(
		// 	0.0,
		// 	0.0,
		// 	offset_strength * 10.0,
		// 	1.0
		// );
}

