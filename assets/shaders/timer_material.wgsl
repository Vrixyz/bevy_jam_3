const PI: f32 = 3.14;
const TWO_PI: f32 = 6.28;

struct TimerMaterial {
    color: vec4<f32>,
    progress: f32
};

@group(1) @binding(0)
var<uniform> material: TimerMaterial;

fn hardCircle(uv: vec2<f32>, radius: f32, width: f32) -> f32 {
    return smoothstep(width, width * 0.99, abs(radius - length(uv)));
}

fn cutSector(uv: vec2<f32>, cutAngle: f32, offset: f32) -> f32 {
    var angle = atan2(uv.y, -uv.x) + PI + offset;
    angle = angle % TWO_PI;//mod(angle, TWO_PI);
    return smoothstep(cutAngle, cutAngle - 0.0001, abs(angle - cutAngle));
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    // var c = material.color;
    // c *= material.progress;
    // c.a = 1.0;
    // return c;
    // Normalized pixel coordinates (from 0 to 1)
    var v_uv = uv - vec2(0.5, 0.5);
    //return vec4<f32>(v_uv, 1.0, 1.0);

    var circle = //hardCircle(v_uv, 0.3, 0.01) + 
    hardCircle(v_uv, 0.4, 0.04) * cutSector(v_uv, TWO_PI * 0.5 * abs(material.progress), TWO_PI * 0.25);

    if circle <= 0.0 {
      discard;
    }
    var col = vec4<f32>(circle * (1.0 - material.progress), circle * material.progress, 0.0, 1.0);
    // Output to screen
    return col;
}