// All WGSL shader code lives here
// Add new shaders (dolphin, sky, etc.) as named exports

export const oceanShader: string = `
  struct Uniforms { mvp: mat4x4<f32> }
  @group(0) @binding(0) var<uniform> u: Uniforms;

  struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color_y: f32,  // ← slow SWE height for coloring
  }

  @vertex
  fn vs_main(
  @location(0) pos: vec3<f32>, 
  @location(1) color_y: f32   // ← 4th attribute
  ) -> VertexOut {
    var out: VertexOut;
    out.pos    = u.mvp * vec4<f32>(pos, 1.0);
    out.color_y = color_y;
    return out;
  }

  @fragment
  fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let t       = clamp((in.color_y + 1.0) / 2.0, 0.0, 1.0);
    let deep    = vec3<f32>(0.0, 0.1, 0.4);
    let shallow = vec3<f32>(0.0, 0.8, 1.0);
    return vec4<f32>(mix(deep, shallow, t), 1.0);
  }
`;
