#pragma once

#include <cstdio>

static constexpr uint16_t REG_IP = 0x0;
static constexpr uint16_t REG_SP = 0x2;
static constexpr uint16_t REG_BP = 0x4;
static constexpr uint16_t REG_SC0 = 0x6;
static constexpr uint16_t REG_SC1 = 0x8;
static constexpr uint16_t REG_SC2 = 0xA;
static constexpr uint16_t REG_SC3 = 0xC;
static constexpr uint16_t STDOUT = 0xE;
static constexpr uint16_t STDIN = 0xF;

class Memory {
    static constexpr size_t MEM_SIZE = 0x10000; // 64k
public:
    static constexpr size_t CODE_START = 1 << 10;
    static constexpr size_t code_start() {
      return CODE_START;
    }

    Memory(const uint8_t* ptr, size_t len): mem_{} {
        if (len < MEM_SIZE - CODE_START) {
            memcpy(&mem_[CODE_START], ptr, len);
        } else {
            fprintf(stderr, "Attempted to load a file that was too big\n");
            std::terminate();
        }
        store16(REG_IP, CODE_START);
        store16(REG_SP, 0x200);
        store16(REG_BP, 0x200);
    };
    // access methods
    uint16_t load16(uint16_t idx) const {
        if (idx % 2 != 0) {
            fprintf(stderr, "Exception: unaligned read (%x)\n", idx);
            std::terminate();
        }
        return mem_[idx] | (mem_[idx + 1] << 8);
    }
    void store16(uint16_t idx, uint16_t load) {
        if (idx % 2 != 0) {
            fprintf(stderr, "Exception: unaligned write (%x)\n", idx);
            std::terminate();
        }
        mem_[idx] = load & 255;
        mem_[idx + 1] = (load >> 8) & 255;
    }
    uint8_t load8(uint16_t idx) const {
        if (idx == STDIN) {
            return getchar();
        }
        return mem_[idx];
    }
    void store8(uint16_t idx, uint8_t load) {
        if (idx == STDOUT) {
            putchar(load);
        }
        mem_[idx] = load & 255;
    }

    char mem_[MEM_SIZE] = {0};
};
