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
    let mediapipe_include = mediapipe_dir.to_string();

    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=src/wrapper.cpp");
    println!("cargo:rerun-if-env-changed=MEDIAPIPE_DIR");

    // Bazel external dependencies path
    let bazel_external = format!("{}/bazel-mediapipe/external", mediapipe_dir);
    // Bazel generated files path (for .pb.h protobuf headers)
    let bazel_bin = format!("{}/bazel-bin", mediapipe_dir);

    // Compile the C++ wrapper
    cc::Build::new()
        .cpp(true)
        .file("src/wrapper.cpp")
        .include(&mediapipe_include)
        .include(&bazel_bin)
        .include("/usr/local/include")
        .include(format!("{}/com_google_absl", bazel_external))
        .include(format!("{}/com_google_protobuf/src", bazel_external))
        .include(format!(
            "{}/external/com_github_glog_glog/_virtual_includes/glog",
            bazel_bin
        ))
        .include(format!(
            "{}/external/com_github_gflags_gflags/_virtual_includes/gflags",
            bazel_bin
        ))
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
        .clang_arg(format!("-I{}", bazel_bin))
        .clang_arg("-I/usr/local/include")
        .clang_arg(format!("-I{}/com_google_absl", bazel_external))
        .clang_arg(format!("-I{}/com_google_protobuf/src", bazel_external))
        .clang_arg(format!(
            "-I{}/external/com_github_glog_glog/_virtual_includes/glog",
            bazel_bin
        ))
        .clang_arg(format!(
            "-I{}/external/com_github_gflags_gflags/_virtual_includes/gflags",
            bazel_bin
        ))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
