pub fn main() {
    let target = std::env::var("TARGET").expect("No TARGET environment variable defined");
    if target == "wasm32-unknown-emscripten" {
        println!("cargo:rustc-link-arg=-sENVIRONMENT=web");
    }
}
