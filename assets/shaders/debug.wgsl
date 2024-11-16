struct DebugSettings {
    enabled: u32,
    texture_width: u32,
    texture_height: u32,
    step: u32,
}

struct ProfilerUniform {
    num_frametimes: u32,
    list: array<f32>,
}

@group(0) @binding(0)
var texture: texture_storage_2d<rgba16float, write>;

@group(0) @binding(1)
var<storage> profiler_data: ProfilerUniform;

var<push_constant> settings: DebugSettings;

@compute
@workgroup_size(8, 8, 1)
fn compute(
    @builtin(local_invocation_id)
    local_id: vec3<u32>,
    @builtin(global_invocation_id)
    global_id: vec3<u32>,
) {
    if settings.enabled == 0 || global_id.x >= settings.texture_width || global_id.y >= settings.texture_height {
        return;
    }

    let width = settings.texture_width;
    let height = settings.texture_height;
    let resolution = vec2(f32(width), f32(height));
    let aspect_ratio = f32(width) / f32(height);

    var texcoord = vec2(f32(global_id.x), f32(global_id.y)) / resolution;
    texcoord.x = 1.0 - texcoord.x;
    texcoord.y = 1.0 - texcoord.y;
    texcoord.y *= aspect_ratio;

    let scale = vec2(4.0, 60.0);
    let tolerance = 1.0 / resolution;

    var color = vec3(0.0);
    var previous_point = vec2(-0.1, 16.666 / scale.y);

    let baseline = 16.666 / scale.y;
    if abs(texcoord.y - baseline) < tolerance.y {
        color = vec3(0.0, 1.0, 0.0);
    }

    for (var i = 0u; i < profiler_data.num_frametimes; i++) {
        let frametime = profiler_data.list[i];

        let x = f32(i) / f32(settings.step) / scale.x;
        let y = frametime / scale.y;
        
        let line = vec2(x, y) - previous_point;
        let line_length = length(line);
        let normalized_line = line / line_length;

        let texcoord_to_previous = texcoord - previous_point;

        let projection_length = dot(texcoord_to_previous, normalized_line);
        let closest_point = previous_point + normalized_line * clamp(projection_length, 0.0, line_length);

        if all(abs(texcoord - closest_point) < tolerance) {
            color = vec3(1.0, 0.0, 0.0);
        }

        previous_point = vec2(x, y);
    }


    textureStore(texture, global_id.xy, vec4(color, 1.0));
}