cmake_minimum_required(VERSION 3.14)
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../cmake")
project(analog_plugin_c C)

#set(USE_LOCAL_HIDAPI YES)
set(HIDAPI_USE_UDEV_HIDRAW YES)
set(CMAKE_C_STANDARD 99)

add_subdirectory(includes)
if(NOT TARGET hidapi)
    message(FATAL_ERROR "This app doesn't do anything without HIDAPI - fix the errors above!")
endif()

add_library(analog_plugin_c SHARED src/plugin.c src/plugin.h)
target_link_libraries(analog_plugin_c PRIVATE hidapi)

