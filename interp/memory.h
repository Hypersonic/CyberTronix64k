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
#define DISPLAY_CONTROL_REG_LOC (DISPLAY_REGION_START-1)

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
