# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    local cargo=cargo
    local test_command=test
    if [ $TRAVIS_OS_NAME = linux ] || [ $TRAVIS_OS_NAME = osx ]; then
      #cargo=cross
      test_command=test-flow
    else
      cargo make pre-test
    fi

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cargo make $test_command
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
