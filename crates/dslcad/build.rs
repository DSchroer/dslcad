pub fn main() {
    let target = std::env::var("TARGET").expect("No TARGET environment variable defined");
    if target == "wasm32-unknown-emscripten" {
        println!("cargo:rustc-link-arg=-sEXPORTED_FUNCTIONS=[_server, _new_buffer, _drop_buffer]");
        println!("cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=[addFunction]");
        println!("cargo:rustc-link-arg=-sENVIRONMENT=web");
        println!("cargo:rustc-link-arg=-sERROR_ON_UNDEFINED_SYMBOLS=0");
        println!("cargo:rustc-link-arg=-sRESERVED_FUNCTION_POINTERS=1");
        println!("cargo:rustc-link-arg=-sTOTAL_STACK=2MB");
    }
}
