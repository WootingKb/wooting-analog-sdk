use std::{env, fs, io, path::PathBuf, process::Command};

const TEST_PLUGIN_DIR: &str = "test_c_plugin";

fn main() -> io::Result<()> {
    println!("cargo::rerun-if-env-changed=WOOTING_BUILD_TEST_PLUGIN");

    // Build the test plugin when WOOTING_BUILD_TEST_PLUGIN=1 is set.
    // CI can use: WOOTING_BUILD_TEST_PLUGIN=1 cargo test
    if env::var("WOOTING_BUILD_TEST_PLUGIN").is_ok() {
        build_and_install_test_plugin()?;
    }

    Ok(())
}

fn build_and_install_test_plugin() -> io::Result<()> {
    if !cmake_available() {
        println!("cargo::warning=cmake not found, skipping test plugin build");
        println!("cargo::warning=Install cmake to enable C plugin tests");
        return Ok(());
    }

    let build_dir = build_test_plugin()?;

    // Determine the library file name based on target OS
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let (lib_name, build_subdir) = match target_os.as_str() {
        "windows" => ("analog_plugin_c.dll", Some("Debug")),
        "macos" => ("libanalog_plugin_c.dylib", None),
        "linux" => ("libanalog_plugin_c.so", None),
        _ => {
            println!("cargo::warning=Unsupported target OS for test plugin: {target_os}");
            return Ok(());
        }
    };

    // Find the built library
    let lib_path = match build_subdir {
        Some(subdir) => build_dir.join(subdir).join(lib_name),
        None => build_dir.join(lib_name),
    };

    if !lib_path.exists() {
        println!("cargo::warning=Test plugin not found at {lib_path:?}, skipping copy");
        return Ok(());
    }

    // Create the test plugins directory in OUT_DIR
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"));
    let plugins_dir = out_dir.join("WootingAnalogPlugins");
    fs::create_dir_all(&plugins_dir)?;

    // Copy the plugin to the plugins directory
    let dest_path = plugins_dir.join(lib_name);
    fs::copy(&lib_path, &dest_path)?;
    println!("cargo::warning=Copied test plugin to {dest_path:?}");

    // Set the environment variable for the test binary so it knows where to find plugins
    println!(
        "cargo::rustc-env=WOOTING_ANALOG_SDK_PLUGINS_PATH={}",
        plugins_dir.display()
    );

    Ok(())
}

fn build_test_plugin() -> io::Result<PathBuf> {
    println!("cargo::warning=Building Test C Plugin");

    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("crate should always have a manifest directory"),
    );
    let source_dir = manifest_dir.join(TEST_PLUGIN_DIR);
    let build_dir = source_dir.join("build");

    fs::create_dir_all(&build_dir)?;

    cmake::Config::new(&source_dir)
        .no_build_target(true)
        .out_dir(&source_dir)
        .always_configure(false)
        .profile("Debug")
        .build();

    Ok(build_dir)
}

fn cmake_available() -> bool {
    Command::new("cmake")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
