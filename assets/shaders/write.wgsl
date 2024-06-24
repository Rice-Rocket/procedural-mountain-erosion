@group(0) @binding(0)
var map: texture_storage_2d<rgba32float, read_write>;

@compute @workgroup_size(8, 8, 1)
fn prepare(@builtin(global_invocation_id) id: vec3<u32>) {
    let original = textureLoad(map, id.xy);
    textureStore(map, id.xy, vec4(original.x));
}
