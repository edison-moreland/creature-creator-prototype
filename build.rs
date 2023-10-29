use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    compile_shader(&PathBuf::from("src/renderer/sphere_shader.metal"));
    compile_shader(&PathBuf::from("src/renderer/widget_shader.metal"));
}

fn compile_shader(shader_source: &Path) {
    println!("cargo:rerun-if-changed={}", shader_source.to_str().unwrap());

    let shader_name = shader_source
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split_once('.')
        .unwrap()
        .0;

    let out_dir = shader_source.parent().unwrap();
    let intermediate_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let air_path = intermediate_dir.join(format!("{}.air", shader_name));
    let metallib_path = out_dir.join(format!("{}.metallib", shader_name));

    panic_if_failed(Command::new("xcrun").args([
        "-sdk",
        "macosx",
        "metal",
        "-gline-tables-only",
        "-frecord-sources",
        "-c",
        shader_source.to_str().unwrap(),
        "-o",
        air_path.to_str().unwrap(),
    ]));

    panic_if_failed(Command::new("xcrun").args([
        "-sdk",
        "macosx",
        "metallib",
        air_path.to_str().unwrap(),
        "-o",
        metallib_path.to_str().unwrap(),
    ]));
}

fn panic_if_failed(command: &mut Command) {
    let output = command.spawn().unwrap().wait_with_output().unwrap();

    if !output.status.success() {
        panic!(
            r#"
stdout: {}
stderr: {}
"#,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        );
    }
}
