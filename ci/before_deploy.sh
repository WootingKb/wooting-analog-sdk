# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) 
          stage=
          lib_ext=
          lib_prefix=
          shared_lib_ext=
          exe_ext=
          cargo=cargo

    case $RUNNER_OS in
        Linux)
            stage=$(mktemp -d)
            lib_ext="a"
            lib_prefix="lib"
            shared_lib_ext="so"
            exe_ext=""
            ;;
        macOS)
            stage=$(mktemp -d -t tmp)
            lib_ext="a"
            lib_prefix="lib"
            shared_lib_ext="dylib"
            exe_ext=""
            ;;
        Windows)
            stage=$(mktemp -d)
            lib_ext="lib"
            lib_prefix=""
            shared_lib_ext="dll"
            exe_ext=".exe"
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # Currently the --out-dir flag is 'unstable' so unfortunately need to switch to nightly for the build to work properly 
    # Don't need to use this currently as the rust-toolchain file specifies the rust version to use
    # rustup default nightly
    cargo make build-target-release


    mkdir $stage/plugins
    mkdir $stage/plugins/lib
    mkdir $stage/plugins/includes

    mkdir $stage/wrapper
    mkdir $stage/wrapper/lib
    mkdir $stage/wrapper/includes
    mkdir $stage/wrapper/sdk

    # Copy Plugin items
    cp target/release-artifacts/${lib_prefix}wooting_analog_common.$lib_ext $stage/plugins/lib
    cp target/release-artifacts/${lib_prefix}wooting_analog_plugin_dev.$lib_ext $stage/plugins/lib

    ## Copy c headers
    cp includes/plugin.h $stage/plugins/includes/
    cp includes/wooting-analog-plugin-dev.h $stage/plugins/includes/
    cp includes/wooting-analog-common.h $stage/plugins/includes/
    ## Copy docs
    cp PLUGINS.md $stage/plugins/



    # Copy wrapper items
    cp target/release-artifacts/${lib_prefix}wooting_analog_wrapper.$shared_lib_ext $stage/wrapper/
    cp target/release-artifacts/${lib_prefix}wooting_analog_wrapper.$lib_ext $stage/wrapper/lib/
    cp target/release-artifacts/${lib_prefix}wooting_analog_sdk.$shared_lib_ext $stage/wrapper/sdk/
    cp target/release-artifacts/${lib_prefix}wooting_analog_test_plugin.$shared_lib_ext $stage/wrapper/sdk/
    # Include Wooting Plugin & Virtual Keyboard app
    cp target/release-artifacts/${lib_prefix}wooting_analog_plugin.$shared_lib_ext $stage/wrapper/sdk/
    cp target/release-artifacts/wooting-analog-virtual-control$exe_ext $stage/wrapper/sdk/

    ## Copy c headers
    cp includes/wooting-analog-wrapper.h $stage/wrapper/includes/
    cp includes/wooting-analog-common.h $stage/wrapper/includes/

    ## Copy docs
    cp SDK_USAGE.md $stage/wrapper/

    # TODO Update this to package the right artifacts
    #cp target/$TARGET/release/hello $stage/

    cd $stage
    tar czf $src/wooting-analog-sdk-$GITHUB_REF_NAME-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
