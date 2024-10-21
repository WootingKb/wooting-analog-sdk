const TEST_PLUGIN_DIR: &str = "test_c_plugin";

fn main() {
    println!("Building Test C Plugin");
    println!("{:?}", std::env::var("OUT_DIR"));

    cmake::Config::new(TEST_PLUGIN_DIR)
        .no_build_target(true)
        .out_dir(format!("./{}", TEST_PLUGIN_DIR))
        .always_configure(false)
        .profile("Debug")
        .build();
}
