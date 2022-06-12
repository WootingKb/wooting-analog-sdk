cmake_minimum_required(VERSION 3.4)
project(test_plugin_c C)

set(CMAKE_C_STANDARD 99)
set(analog_sdk_path ${CMAKE_CURRENT_SOURCE_DIR}/../../)
link_directories($ENV{OUT_DIR}/../../../)

add_library(analog_plugin_c SHARED src/plugin.c)
if (WIN32)
    # On Windows bcrypt needs to be linked as well for the rust libs
    target_link_libraries(analog_plugin_c PRIVATE bcrypt)
endif (WIN32)
target_link_libraries(analog_plugin_c PRIVATE wooting_analog_plugin_dev)
target_link_libraries(analog_plugin_c PRIVATE wooting_analog_common)