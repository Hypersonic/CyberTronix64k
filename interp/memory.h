#pragma once
#include <cstdio>
#include <atomic>

typedef uint16_t mem_t;

#define PRINT_LOC 0x5


class memory {
public:
    memory() : mem{0} {};
    // access methods
    mem_t& operator[](size_t idx) {
        if (idx == PRINT_LOC) {
            must_print = true;
            return mem[idx];
        }
        return mem[idx];
    }

    void flush_changes() {
        if (must_print) {
            putchar(char(mem[PRINT_LOC]));
            must_print = false;
        }
    }

    bool must_print;
    mem_t mem[1<<15] = {0}; // lookups that don't hit mmapped regions hit this
};
