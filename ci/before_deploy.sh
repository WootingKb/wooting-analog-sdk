# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) 
          stage=
          lib_ext=
          lib_prefix=
          shared_lib_ext=
          cargo=cargo

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            lib_ext="a"
            lib_prefix="lib"
            shared_lib_ext="so"
            cargo=cross
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            lib_ext="a"
            lib_prefix="lib"
            shared_lib_ext="dylib"
            cargo=cross
            ;;
        windows)
            stage=$(mktemp -d)
            lib_ext="lib"
            lib_prefix=""
            shared_lib_ext="dll"
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # TODO Update this to build the artifacts that matter to you
    cargo make build -e CARGO_COMMAND=$cargo -- --target $TARGET --release


    mkdir $stage/plugins
    mkdir $stage/plugins/lib
    mkdir $stage/plugins/includes
    mkdir $stage/plugins/includes-cpp

    mkdir $stage/wrapper
    mkdir $stage/wrapper/includes
    mkdir $stage/wrapper/includes-cpp
    mkdir $stage/wrapper/sdk

    # Copy Plugin items
    cp target/$TARGET/release/${lib_prefix}wooting_analog_common.$lib_ext $stage/plugins/lib
    cp target/$TARGET/release/${lib_prefix}wooting_analog_plugin_dev.$lib_ext $stage/plugins/lib

    ## Copy c headers
    cp includes/plugin.h $stage/plugins/includes/
    cp includes/wooting-analog-plugin-dev.h $stage/plugins/includes/
    cp includes/wooting-analog-common.h $stage/plugins/includes/

    ## Copy cpp headers
    cp includes-cpp/wooting-analog-plugin-dev.h $stage/plugins/includes-cpp/
    cp includes-cpp/wooting-analog-common.h $stage/plugins/includes-cpp/

    ## Copy docs
    cp PLUGINS.md $stage/plugins/



    # Copy wrapper items
    cp target/$TARGET/release/${lib_prefix}wooting_analog_wrapper.$shared_lib_ext $stage/wrapper/
    cp target/$TARGET/release/${lib_prefix}wooting_analog_sdk.$shared_lib_ext $stage/wrapper/sdk/
    ls wooting-analog-test-plugin/target
    ls wooting-analog-test-plugin/target/release
    ls wooting-analog-test-plugin/target/$TARGET/release
    cp wooting-analog-test-plugin/target/$TARGET/release/${lib_prefix}wooting_analog_test_plugin.$shared_lib_ext $stage/wrapper/sdk/

    ## Copy c headers
    cp includes/wooting-analog-wrapper.h $stage/wrapper/includes/
    cp includes/wooting-analog-common.h $stage/wrapper/includes/

    ## Copy cpp headers
    cp includes-cpp/wooting-analog-wrapper.h $stage/wrapper/includes-cpp/
    cp includes-cpp/wooting-analog-common.h $stage/wrapper/includes-cpp/

    ## Copy docs
    cp SDK_USAGE.md $stage/wrapper/

    # TODO Update this to package the right artifacts
    #cp target/$TARGET/release/hello $stage/

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
