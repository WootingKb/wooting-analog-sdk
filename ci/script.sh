# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    local cargo=cargo
    local test_command=test
    if [ $TRAVIS_OS_NAME = linux ] || [ $TRAVIS_OS_NAME = osx ]; then
      cargo=cross
      test_command=test-flow
    fi

    #cross build --target $TARGET
    #cross build --target $TARGET --release
    cargo make build -e CARGO_COMMAND=$cargo -- --target $TARGET --release


    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi
    
    cargo make $test_command -e CARGO_COMMAND=$cargo -- --target $TARGET
    #cross test --target $TARGET
    #cross test --target $TARGET --release

    #cross run --target $TARGET
    #cross run --target $TARGET --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
