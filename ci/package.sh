# This script takes care of packaging the SDK for release

set -ex

main() {
    local stage=
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
            platform="linux"
            ;;
        macOS)
            stage=$(mktemp -d -t tmp)
            lib_ext="a"
            lib_prefix="lib"
            shared_lib_ext="dylib"
            exe_ext=""
            platform="mac"
            ;;
        Windows)
            stage=$(mktemp -d)
            lib_ext="lib"
            lib_prefix=""
            shared_lib_ext="dll"
            exe_ext=".exe"
            platform="win"
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    mkdir $stage/debug
    mkdir $stage/release
    mkdir $stage/includes

    ## Copy docs
    cp PLUGINS.md $stage/
    cp SDK_USAGE.md $stage/

    # sdk build artifacts
    cp target/debug/${lib_prefix}wooting_analog_sdk.$shared_lib_ext $stage/debug/
    cp target/release/${lib_prefix}wooting_analog_sdk.$shared_lib_ext $stage/release/
    if [ $RUNNER_OS = Windows ]; then
        cp target/debug/${lib_prefix}wooting_analog_sdk.$shared_lib_ext.lib $stage/debug/
        cp target/release/${lib_prefix}wooting_analog_sdk.$shared_lib_ext.lib $stage/release/
    fi

    # dist build artifacts
    cp target/debug/${lib_prefix}wooting_analog_sdk_dist.$shared_lib_ext $stage/debug/
    cp target/release/${lib_prefix}wooting_analog_sdk_dist.$shared_lib_ext $stage/release/
    if [ $RUNNER_OS = Windows ]; then
        cp target/debug/${lib_prefix}wooting_analog_sdk_dist.$shared_lib_ext.lib $stage/debug/
        cp target/release/${lib_prefix}wooting_analog_sdk_dist.$shared_lib_ext.lib $stage/release/
    fi
    
    cp target/release/wooting-analog-virtual-control$exe_ext $stage/

    ## Copy c headers
    cp includes/plugin.h $stage/includes
    cp includes/wooting-analog-sdk.h $stage/includes

    if [ $RUNNER_OS = Windows ]; then
        (cd "$stage" && 7z a "$GITHUB_WORKSPACE/wooting-analog-sdk-$VERSION-$TARGET.zip" *)
    else
        (cd "$stage" && tar czf "$GITHUB_WORKSPACE/wooting-analog-sdk-$VERSION-$TARGET.tar.gz" *)
    fi

    rm -rf $stage
}

main
