cmake_minimum_required(VERSION 3.4)
project(test_plugin_c C)

set(CMAKE_C_STANDARD 99)
set(analog_sdk_path ${CMAKE_CURRENT_SOURCE_DIR}/../../)

add_library(analog_plugin_c SHARED src/plugin.c)
#target_link_directories(analog_plugin_c PRIVATE ${analog_sdk_path}/target/debug)
#target_link_libraries(analog_plugin_c PRIVATE wooting_analog_plugin_dev)