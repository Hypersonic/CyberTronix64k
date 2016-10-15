#pragma once

typedef uint16_t mem_t;

#define DISPLAY_REGION_SIZE 0xFF
#define DISPLAY_REGION_START 0x100
#define DISPLAY_REGION_END (DISPLAY_REGION_START + DISPLAY_REGION_SIZE)
#define DISPLAY_CONTROL_REG_LOC (DISPLAY_REGION_START-1)

class display {
public:
    void redraw() {}; // TODO
    mem_t disp_coords;
    mem_t screen[DISPLAY_REGION_SIZE];
};

class memory {
public:
    // access methods
    mem_t& operator[](size_t idx) {
        if (DISPLAY_CONTROL_REG_LOC == idx) {
            return disp.disp_coords;
        }
        if (DISPLAY_REGION_START <= idx && idx < DISPLAY_REGION_END) {
            return disp.screen[idx - DISPLAY_REGION_SIZE];
        }
        return mem[idx];
    }

    display disp;
    mem_t mem[1<<15] = {0}; // lookups that don't hit mmapped regions hit this
};
