#pragma once

typedef uint16_t mem_t;

class Memory {
public:
    mem_t& operator[](size_t idx) {
        // TODO: mmaped regions
        return mem[idx];
    }
private:
    mem_t mem[1<<15] = {0}; // lookups that don't hit mmapped regions hit this
};
