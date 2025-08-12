fn complex_square(z: vec2f) -> vec2f {
  return vec2f(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y);
}

fn julia(z: ptr<function, vec2f>, c: vec2f) -> f32 {
  const max_iterations = 128;
  var n: f32 = 0;
  for (var i: u32 = 0; i < max_iterations; i++) {
    // Mandelbrot/Julia formula
    * z = complex_square(*z) + c;
    if dot(*z, * z) > 512.0 {
      break;
    }
    n = 1;
  }

  if u32(n) == max_iterations {
    return 0;
  }

  let sn = n - log2(log2(dot(*z, * z))) + 4.0;
  return max(sn, 1);
}

fn hueShift(color: vec3f, u_theta: f32) -> vec3f {
  //Adapted from https://www.reddit.com/r/gamemaker/comments/1bq0lul/a_hueshifting_shader_that_preserves_colors/
  var yiqColor = mat3x3(0.299, 1, 0.40462981, 0.587, - 0.46081557, - 1, 0.114, - 0.53918443, 0.59537019) * color.rgb;
  yiqColor = vec3f(0, mat2x2(cos(u_theta), sin(u_theta), - sin(u_theta), cos(u_theta)) * yiqColor.yz);
  return mat3x3(1, 1, 1, 0.5696804, - 0.1620848, - 0.6590654, 0.3235513, - 0.3381869, 0.8901581) * yiqColor;
}

@group(0) @binding(0)
var<uniform> time: f32;

@group(0) @binding(1)
var<uniform> resolution: vec2f;

@fragment
fn fs_main(@builtin(position) fragCoord: vec4f) -> @location(0) vec4f {
  let uv = fragCoord.xy / resolution;

  return vec4f(uv, 0.5 + 0.5 * cos(time), 1);
}

@vertex
fn vs_main(@location(0) vertexCoord: vec2f) -> @builtin(position) vec4f {
  return vec4f(vertexCoord, 0, 1);
}
