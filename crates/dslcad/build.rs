pub fn main() {
    let target = std::env::var("TARGET").expect("No TARGET environment variable defined");
    if target == "wasm32-unknown-emscripten" {
        println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH");
        println!("cargo:rustc-link-arg=-sENVIRONMENT=web");
        println!("cargo:rustc-link-arg=-sMODULARIZE");
        println!("cargo:rustc-link-arg=-sEXPORT_NAME=dslcad");
        println!("cargo:rustc-link-arg=-sEXPORT_ES6=1");
        println!("cargo:rustc-link-arg=-sSTACK_SIZE=10485760");
        println!("cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=FS,callMain");
    }
}
