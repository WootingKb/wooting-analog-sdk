#include "plugin.h"

const char* c_name(){
    return "C Test Plugin";
}

void on_plugin_load() {
    printf("%s Plugin loaded\n", c_name());
}
 
void on_plugin_unload(){
    printf("%s Plugin unloaded\n", c_name());
}

uint32_t add(uint32_t x, uint32_t y) {
    return 22;
}