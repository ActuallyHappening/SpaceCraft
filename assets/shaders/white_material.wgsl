#import bevy_pbr::forward_io::VertexOutput
// we can import items from shader modules in the assets folder with a quoted path
// #import "shaders/custom_material_import.wgsl"::COLOR_MULTIPLIER
// const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 0.7);

#import bevy_pbr::mesh_view_bindings::globals;

struct CustomMaterial {
    color: vec4<f32>,
};

// @group(1) @binding(0) var<uniform> material: CustomMaterial;
// @group(1) @binding(1) var base_color_texture: texture_2d<f32>;
// @group(1) @binding(2) var base_color_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    // return material.color * textureSample(base_color_texture, base_color_sampler, mesh.uv);
		// return vec4<f32>(1.0, 0.0, 0.0, 1.0);
		// return material.color;
		return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
