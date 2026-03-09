fn main() {
    // Add SDL2 library search path
    let sdl2_lib_dir = std::env::var("SDL2_LIB_DIR").unwrap_or_else(|_| "lib".to_string());
    println!("cargo:rustc-link-search=native={}", sdl2_lib_dir);
}
