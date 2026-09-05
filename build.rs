fn main() {
    // wgpu loads WGSL shaders at runtime - no build-time compilation needed.
    // The build script is kept for potential future use (e.g., asset embedding).
    println!("cargo:rerun-if-changed=assets/shaders/");
}
