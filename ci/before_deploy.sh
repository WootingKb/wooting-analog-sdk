# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) 
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # TODO Update this to build the artifacts that matter to you
    cargo make default -e CARGO_COMMAND=cross -- --target $TARGET --release


    # Copy Plugin items
    cp target/$TARGET/release/libwooting_analog_common.a $stage/plugins/lib

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
    cp target/$TARGET/release/libwooting_analog_wrapper.so $stage/wrapper/
    cp target/$TARGET/release/libwooting_analog_sdk.so $stage/wrapper/sdk/

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
