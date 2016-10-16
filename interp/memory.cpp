#include "memory.h"

void display::redraw() {
    if (!is_init) {
        // setup
        if (SDL_Init(SDL_INIT_VIDEO) != 0) {
            abort();
        }

        window = SDL_CreateWindow("CYBERTRONIX-64K", 100, 100, DISPLAY_DIMENSIONS, DISPLAY_DIMENSIONS, SDL_WINDOW_SHOWN);
        if (!window) {
            abort();
        }

        renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC);
        if (!renderer) {
            abort();
        }
        is_init = true;
    }

    // eat SDL events
    SDL_Event event;
    while (SDL_PollEvent(&event)) {
        switch (event.type) {
            case SDL_QUIT:
                exit(0); // we could cleanup but w/e
                break;
            default:
                break;
        }
    }
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 0);
    SDL_RenderClear(renderer);

    uint8_t base_x, base_y;
    // Render
    for (size_t y = 0; y < DISPLAY_DIMENSIONS; ++y) {
        for (size_t x = 0; x < DISPLAY_DIMENSIONS; ++x) {
            uint8_t r,g,b,a;
            size_t offset = (y * DISPLAY_DIMENSIONS) + x;
            mem_t pixel = screen[offset];
            a = ((pixel & 0xF000) >> 12) * 16;
            b = ((pixel & 0x0F00) >> 8 ) * 16;
            g = ((pixel & 0x00F0) >> 4 ) * 16;
            r = ((pixel & 0x000F) >> 0 ) * 16;

            SDL_SetRenderDrawColor(renderer, r, g, b, a);
            SDL_RenderDrawPoint(renderer, x, y);
        }
    }

    SDL_RenderPresent(renderer);
};
