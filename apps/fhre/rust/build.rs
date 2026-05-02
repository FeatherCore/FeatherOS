use std::{
    env,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

fn main() {
    println!("cargo:rerun-if-changed=src/egl_shim.c");
    println!("cargo:rerun-if-env-changed=FHRE_SIM_OPENGL");
    println!("cargo:rerun-if-env-changed=FHRE_EGL_WORKER_ENABLE");

    if env::var_os("CARGO_FEATURE_SIM_OPENGL_EGL").is_none() {
        return;
    }
    println!("cargo:rustc-check-cfg=cfg(fhre_real_egl_gl)");

    let egl_worker_requested = env::var("FHRE_SIM_OPENGL")
        .map(|value| value == "egl")
        .unwrap_or(false)
        || env::var("FHRE_EGL_WORKER_ENABLE")
            .map(|value| value == "1")
            .unwrap_or(false);

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let object = out_dir.join("fhre_egl_shim.o");
    let archive = out_dir.join("libfhre_egl_shim.a");
    let probe = out_dir.join("fhre_egl_probe.c");
    let probe_bin = out_dir.join("fhre_egl_probe");
    let cc = env::var_os("CC").unwrap_or_else(|| "cc".into());
    let ar = env::var_os("AR").unwrap_or_else(|| "ar".into());

    let probe_source = r#"
#include <EGL/egl.h>
#include <GL/gl.h>
int main(void) {
    (void)EGL_NO_DISPLAY;
    (void)GL_RGBA;
    return 0;
}
"#;
    fs::write(&probe, probe_source).expect("failed to write fhre EGL probe");
    let egl_gl_available = egl_worker_requested
        && Command::new(&cc)
            .arg("-std=c99")
            .arg(&probe)
            .arg("-lEGL")
            .arg("-lGL")
            .arg("-lpthread")
            .arg("-o")
            .arg(&probe_bin)
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false);

    if egl_gl_available {
        println!("cargo:rustc-cfg=fhre_real_egl_gl");
        println!("cargo:rustc-link-lib=dylib=EGL");
        println!("cargo:rustc-link-lib=dylib=GL");
        println!("cargo:rustc-link-lib=dylib=pthread");
    } else if egl_worker_requested {
        println!("cargo:warning=FHRE sim-opengl-egl built without host EGL/OpenGL headers/libs; EGL worker will report unavailable");
    }

    let cc_status = Command::new(&cc)
        .arg("-std=c99")
        .arg("-Wall")
        .arg("-Wextra")
        .args(if egl_gl_available {
            &["-DFHRE_EGL_REAL=1"][..]
        } else {
            &[][..]
        })
        .arg("-c")
        .arg("src/egl_shim.c")
        .arg("-o")
        .arg(&object)
        .stderr(Stdio::inherit())
        .status()
        .expect("failed to launch C compiler for fhre EGL shim");
    assert!(cc_status.success(), "failed to compile fhre EGL shim");

    let ar_status = Command::new(&ar)
        .arg("crs")
        .arg(&archive)
        .arg(&object)
        .stderr(Stdio::inherit())
        .status()
        .expect("failed to launch archiver for fhre EGL shim");
    assert!(ar_status.success(), "failed to archive fhre EGL shim");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=fhre_egl_shim");
}
