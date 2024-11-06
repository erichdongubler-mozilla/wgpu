@group(0) @binding(0)
var writeTex: texture_storage_2d<rgba8unorm, write>;

fn write_black_to_storage_texture(
  // writeTex: texture_storage_2d<rgba8unorm, write>, // BUG: uncomment this, and this won't compile
) {
  textureStore(writeTex, vec2(0), vec4(0.));
}
