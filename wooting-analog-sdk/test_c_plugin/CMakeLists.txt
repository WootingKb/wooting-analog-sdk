cmake_minimum_required(VERSION 3.4)
project(test_plugin_c C)

set(CMAKE_C_STANDARD 99)
set(analog_sdk_path ${CMAKE_CURRENT_SOURCE_DIR}/../../)
link_directories($ENV{OUT_DIR}/../../../)

add_library(analog_plugin_c SHARED src/plugin.c)
