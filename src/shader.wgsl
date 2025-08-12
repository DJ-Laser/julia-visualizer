fn complex_square(z: vec2f) -> vec2f {
  return vec2f(z.x * z.x - z.y * z.y, 2 * z.x * z.y);
}

fn julia(initial_z: vec2f, c: vec2f) -> f32 {
  const max_iterations: f32 = 128;

  var z = initial_z;
  var n: f32 = 0;

  for (; n < max_iterations; n += 1) {
    // Mandelbrot/Julia formula
    z = complex_square(z) + c;

    if dot(z, z) > 512 {
      break;
    }
  }

  if n == max_iterations {
    return 0;
  }

  let sn = n - log2(log2(dot(z, z))) + 4;
  return max(sn, 1);
}

fn hue_shift(color: vec3f, theta: f32) -> vec3f {
  //Adapted from https://www.reddit.com/r/gamemaker/comments/1bq0lul/a_hueshifting_shader_that_preserves_colors/
  var intermediate_color = (mat3x3f(vec3f(0.299, 1, 0.4046298), vec3f(0.587, - 0.46081558, - 1), vec3f(0.114, - 0.53918445, 0.5953702)) * color.xyz);
  let res = (mat2x2f(vec2f(cos(theta), sin(theta)), vec2f(- (sin(theta)), cos(theta))) * intermediate_color.yz);
  intermediate_color.y = res.x;
  intermediate_color.z = res.y;
  return (mat3x3f(vec3f(1, 1, 1), vec3f(0.5696804f, - 0.1620848f, - 0.6590654f), vec3(0.3235513f, - 0.3381869f, 0.8901581f)) * intermediate_color);
}

@group(0) @binding(0)
var<uniform> time: f32;

@group(0) @binding(1)
var<uniform> resolution: vec2f;

@fragment
fn fs_main(@builtin(position) fragCoord: vec4f) -> @location(0) vec4f {
  let uv = fragCoord.xy / resolution;

  // Make julia apear centered on screen
  let juliaUv = (fragCoord.xy / resolution.y - vec2f(0.5 * resolution.x / resolution.y, 0.5)) * 2;
  let c = vec2f(- 0.5 * cos(time / 11), - 0.2 * sin(time / 7));

  let julia = julia(juliaUv, c);
  var col = 0.5 + 0.5 * cos(3 + julia * 0.15 + vec3f(0, 2, 4));

  if (julia < 0.5) {
    col = vec3f(1, 1, 1);
  }

  col = hue_shift(col, time * 2.0);

  return vec4f(col, 1);
}

@vertex
fn vs_main(@location(0) vertexCoord: vec2f) -> @builtin(position) vec4f {
  return vec4f(vertexCoord, 0, 1);
}
