use std::env;
use std::path::PathBuf;

fn main() {
    // Get MediaPipe installation path from environment variable
    let mediapipe_dir =
        env::var("MEDIAPIPE_DIR").unwrap_or_else(|_| "/usr/local/mediapipe".to_string());

    let mediapipe_lib = format!(
        "{}/bazel-bin/mediapipe/examples/desktop/hand_tracking",
        mediapipe_dir
    );
    let mediapipe_include = format!("{}", mediapipe_dir);

    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=src/wrapper.cpp");
    println!("cargo:rerun-if-env-changed=MEDIAPIPE_DIR");

    // Compile the C++ wrapper
    cc::Build::new()
        .cpp(true)
        .file("src/wrapper.cpp")
        .include(&mediapipe_include)
        .include("/usr/local/include")
        .flag("-std=c++17")
        .flag("-Wno-sign-compare")
        .compile("mediapipe_wrapper");

    // Link MediaPipe libraries
    println!("cargo:rustc-link-search=native={}", mediapipe_lib);
    println!("cargo:rustc-link-search=native=/usr/local/mediapipe/lib");
    println!("cargo:rustc-link-lib=dylib=mediapipe_hand_tracking");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_arg(format!("-I{}", mediapipe_include))
        .clang_arg("-I/usr/local/include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
