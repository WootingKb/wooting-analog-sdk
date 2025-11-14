use std::{fs, io, env, path::PathBuf};

const TEST_PLUGIN_DIR: &str = "test_c_plugin";

fn main() -> io::Result<()> {
    println!("Building Test C Plugin");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_dir = manifest_dir.join(TEST_PLUGIN_DIR);
    let out_dir = source_dir.join("build");

    fs::create_dir_all(out_dir)?;
    
    cmake::Config::new(&source_dir)
        .no_build_target(true)
        .out_dir(&source_dir)
        .always_configure(false)
        .profile("Debug")
        .build();

    Ok(())
}
