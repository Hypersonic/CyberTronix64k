#pragma once
#include <SDL.h>
#include <atomic>

typedef uint16_t mem_t;

#define DISPLAY_DIMENSIONS 255
#define DISPLAY_SIZE (DISPLAY_DIMENSIONS * DISPLAY_DIMENSIONS)
#define DISPLAY_REGION_DIMENSIONS 16
#define DISPLAY_REGION_SIZE (DISPLAY_REGION_DIMENSIONS * DISPLAY_REGION_DIMENSIONS)
#define DISPLAY_REGION_START 0x100
#define DISPLAY_REGION_END (DISPLAY_REGION_START + DISPLAY_REGION_SIZE)
#define DISPLAY_COORD_REG_LOC (DISPLAY_REGION_START-1)
#define DISPLAY_CONTROL_REG_LOC (DISPLAY_REGION_START-2)

#define PRINT_LOC 0x5

class display {
public:
    display() : screen{0}, is_init(false) {};
    void redraw();
    mem_t disp_coords;
    mem_t screen[DISPLAY_SIZE];
    std::atomic<bool> is_init;

private:
    SDL_Window *window;
    SDL_Renderer *renderer;
};

class memory {
public:
    memory() : disp(), mem{0} {};
    // access methods
    mem_t& operator[](size_t idx) {
        if (DISPLAY_COORD_REG_LOC == idx) {
            return disp.disp_coords;
        }
        if (DISPLAY_CONTROL_REG_LOC == idx) {
            return redraw_supressed;
        }
        if (DISPLAY_REGION_START <= idx && idx < DISPLAY_REGION_END) {
            must_update_display = true;
            uint8_t disp_x, disp_y, x, y;
            disp_x = disp.disp_coords & 0x00FF;
            disp_y = (disp.disp_coords & 0xFF00) >> 8;
            uint16_t offset = idx - DISPLAY_REGION_SIZE;
            x = offset % DISPLAY_REGION_DIMENSIONS;
            y = (offset - x) / DISPLAY_REGION_DIMENSIONS;
            offset = ((disp_y + y) * DISPLAY_DIMENSIONS) + (disp_x + x);
            return disp.screen[offset];
        }
        if (idx == PRINT_LOC) {
            must_print = true;
            return mem[idx];
        }
        return mem[idx];
    }

    void flush_changes() {
        if (must_update_display && !redraw_supressed) {
            disp.redraw();
            must_update_display = false;
        }
        if (must_print) {
            putchar(char(mem[PRINT_LOC]));
            must_print = false;
        }
    }

    bool must_print, must_update_display;
    mem_t redraw_supressed; // don't redraw until this is 0.
    display disp;
    mem_t mem[1<<15] = {0}; // lookups that don't hit mmapped regions hit this
};
