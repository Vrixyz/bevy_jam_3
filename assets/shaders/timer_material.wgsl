struct TimerMaterial {
    color: vec4<f32>,
    progress: f32
};

@group(1) @binding(0)
var<uniform> material: TimerMaterial;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    //var c = material.color;
    var c = material.color;
    c *= material.progress;
    c.a = 1.0;
    return c;
}