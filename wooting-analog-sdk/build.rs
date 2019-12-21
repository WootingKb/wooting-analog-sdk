const TEST_PLUGIN_DIR: &str = "test_c_plugin";


fn main() {
    cmake::Config::new(TEST_PLUGIN_DIR).no_build_target(true).out_dir(format!("./{}", TEST_PLUGIN_DIR)).profile("Debug").build();
}