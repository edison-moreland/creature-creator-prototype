use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    compile_shader(&PathBuf::from("src/shader.metal"));
}

// xcrun -sdk macosx metal -c shaders.metal -o shaders.air
// xcrun -sdk macosx metallib shaders.air -o shaders.metallib
fn compile_shader(shader_source: &PathBuf) {
    // TODO: Rewrite all of this
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

    let output = Command::new("xcrun")
        .arg("-sdk")
        .arg("macosx")
        .arg("metal")
        .args(&["-gline-tables-only"])
        .args(&["-frecord-sources"])
        .args(&["-c", shader_source.to_str().unwrap()])
        .args(&["-o", air_path.to_str().unwrap()])
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    if !output.status.success() {
        panic!(
            r#"
            air
stdout: {}
stderr: {}
"#,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        );
    }

    let output = Command::new("xcrun")
        .arg("-sdk")
        .arg("macosx")
        .arg("metallib")
        .arg(&air_path)
        .args(&["-o", metallib_path.to_str().unwrap()])
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    if !output.status.success() {
        panic!(
            r#"
            metallib
stdout: {}
stderr: {}
"#,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        );
    }
}
