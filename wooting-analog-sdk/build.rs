fn main() -> std::io::Result<()> {
    #[cfg(test)]
    build_test_plugin()?;

    Ok(())
}

// TODO: rework this when we get to unit/integration testing
// this fails to compile if you `cargo test` without cmake being installed on the system
#[cfg(test)]
mod test {
    use std::{env, fs, io, path::PathBuf};

    const TEST_PLUGIN_DIR: &str = "test_c_plugin";

    fn build_test_plugin() -> io::Result<()> {
        println!("Building Test C Plugin");

        let manifest_dir = PathBuf::from(
            env::var("CARGO_MANIFEST_DIR").expect("crate should always have a manifest directory"),
        );
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
}
