#include <cstdio>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <cerrno>
#include <exception>

#include <unistd.h>

#include "memory.h"

// macro to print instruction traces, + ip
#ifdef DO_TRACE
#  define P_TRACE(fmt, ...) fprintf(stderr, "0x%04x: " fmt "\n", ip, __VA_ARGS__)
#else
#  define P_TRACE(fmt, ...)
#endif


class Opcode {
  enum class BaseOp: uint8_t {
    Mvi = 0x0,
    Mv = 0x1,
    Mvd = 0x2,
    And = 0x3,
    Or  = 0x4,
    Xor = 0x5,
    Add = 0x6,
    Sub = 0x7,
    Shr = 0x8,
    Shl = 0x9,
    Sha = 0xA,
    Jl  = 0xB,
    Jg  = 0xC,
    Jb  = 0xD,
    Ja  = 0xE,
    Jq  = 0xF
  };

  BaseOp base_;
  bool bits16_;
  bool imm_;

  static constexpr uint16_t get_op(uint16_t inst) {
    return inst >> 10;
  }
  static constexpr uint16_t get_reg(uint16_t inst) {
    return inst & ((1 << 10) - 1);
  }

public:
  Opcode(uint16_t inst) {
    uint16_t opcode = get_op(inst);
    base_ = BaseOp(opcode & 0b001111);
    bits16_ = (opcode & 0b010000) == 0;
    imm_ = (opcode & 0b100000) != 0;
  }

  void operator()(Memory& memory) {
    auto ip = memory.load16(REG_IP);
    auto reg = get_reg(memory.load16(ip));
    auto arg1 = get_reg(memory.load16(ip + 2));
    switch (base_) {
      case BaseOp::Mvi: {
        if (bits16_) {
          if (!imm_) { P_TRACE("mvi 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, arg1);
          } else { P_TRACE("ldi (0x%X), 0x%X", reg, arg1);
            memory.store16(memory.load16(reg), arg1);
          }
        } else {
          if (!imm_) {
            if (reg == 0 && arg1 == 0) {
              P_TRACE("mvib 0x%X, 0x%X (hcf)", reg, arg1);
              printf("HCF instruction reached\n");
              std::exit(0);
            }
            P_TRACE("mvib 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, uint8_t(arg1));
          } else { P_TRACE("ldib 0x%X, 0x%X", reg, arg1);
            memory.store8(memory.load16(reg), uint8_t(arg1));
          }
        }
      } break;
      case BaseOp::Mv: {
        if (bits16_) {
          if (!imm_) { P_TRACE("mv 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(arg1));
          } else { P_TRACE("ld 0x%X, 0x%X", reg, arg1);
            memory.store16(memory.load16(reg), memory.load16(arg1));
          }
        } else {
          if (!imm_) { P_TRACE("mvb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(arg1));
          } else { P_TRACE("ldb (0x%X), 0x%X", reg, arg1);
            memory.store8(memory.load16(reg), memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Mvd: {
        if (bits16_) {
          if (!imm_) { P_TRACE("mvd 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(memory.load16(arg1)));
          } else { P_TRACE("ldd 0x%X, 0x%X", reg, arg1);
            memory.store16(memory.load16(reg),
              memory.load16(memory.load16(arg1)));
          }
        } else {
          if (!imm_) { P_TRACE("mvdb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(memory.load16(arg1)));
          } else { P_TRACE("lddb (0x%X), 0x%X", reg, arg1);
            memory.store8(memory.load16(reg),
              memory.load8(memory.load16(arg1)));
          }
        }
      } break;
      case BaseOp::And: {
        if (bits16_) {
          if (imm_) { P_TRACE("and 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) & arg1);
          } else { P_TRACE("and 0x%X, (0x%X)", reg, arg1);
            memory.store16(reg, memory.load16(reg) & memory.load16(arg1));
          }
        } else {
          if (imm_) { P_TRACE("andb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) & uint8_t(arg1));
          } else { P_TRACE("andb 0x%X, (0x%X)", reg, arg1);
            memory.store8(reg, memory.load8(reg) & memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Or: {
        if (bits16_) {
          if (imm_) { P_TRACE("or 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) | arg1);
          } else { P_TRACE("or 0x%X, (0x%X)", reg, arg1);
            memory.store16(reg, memory.load16(reg) | memory.load16(arg1));
          }
        } else {
          if (imm_) { P_TRACE("orb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) | uint8_t(arg1));
          } else { P_TRACE("orb 0x%X, (0x%X)", reg, arg1);
            memory.store8(reg, memory.load8(reg) | memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Xor: {
        if (bits16_) {
          if (imm_) { P_TRACE("xor 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) ^ arg1);
          } else { P_TRACE("xor 0x%X, (0x%X)", reg, arg1);
            memory.store16(reg, memory.load16(reg) ^ memory.load16(arg1));
          }
        } else {
          if (imm_) { P_TRACE("xorb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) ^ uint8_t(arg1));
          } else { P_TRACE("xorb 0x%X, (0x%X)", reg, arg1);
            memory.store8(reg, memory.load8(reg) ^ memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Add: {
        if (bits16_) {
          if (imm_) { P_TRACE("add 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) + arg1);
          } else { P_TRACE("add 0x%X, (0x%X)", reg, arg1);
            memory.store16(reg, memory.load16(reg) + memory.load16(arg1));
          }
        } else {
          if (imm_) { P_TRACE("addb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) + uint8_t(arg1));
          } else { P_TRACE("addb 0x%X, (0x%X)", reg, arg1);
            memory.store8(reg, memory.load8(reg) + memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Sub: {
        if (bits16_) {
          if (imm_) { P_TRACE("sub 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) - arg1);
          } else { P_TRACE("sub 0x%X, (0x%X)", reg, arg1);
            memory.store16(reg, memory.load16(reg) - memory.load16(arg1));
          }
        } else {
          if (imm_) { P_TRACE("subb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) - uint8_t(arg1));
          } else { P_TRACE("subb 0x%X, (0x%X)", reg, arg1);
            memory.store8(reg, memory.load8(reg) - memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Shr: {
        if (bits16_) {
          if (imm_) { P_TRACE("shr 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) >> arg1 & 15);
          } else { P_TRACE("shr 0x%X, (0x%X)", reg, arg1);
            memory.store16(reg,
              memory.load16(reg) >> memory.load8(arg1) & 15);
          }
        } else {
          if (imm_) { P_TRACE("shrb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) >> arg1 & 7);
          } else { P_TRACE("shrb 0x%X, (0x%X)", reg, arg1);
            memory.store8(reg,
              memory.load8(reg) >> memory.load8(arg1) & 7);
          }
        }
      } break;
      case BaseOp::Shl: {
        if (bits16_) {
          if (imm_) { P_TRACE("shl 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) << arg1 & 15);
          } else { P_TRACE("shl 0x%X, (0x%X)", reg, arg1);
            memory.store16(reg,
              memory.load16(reg) << memory.load8(arg1) & 15);
          }
        } else {
          if (imm_) { P_TRACE("shlb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) << arg1 & 7);
          } else { P_TRACE("shlb 0x%X, (0x%X)", reg, arg1);
            memory.store8(reg,
              memory.load8(reg) << memory.load8(arg1) & 7);
          }
        }
      } break;
      case BaseOp::Sha: {
        if (bits16_) {
          if (imm_) { P_TRACE("sha 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, uint16_t(
                int16_t(memory.load16(reg)) >> arg1 & 15));
          } else { P_TRACE("sha 0x%X, (0x%X)", reg, arg1);
            memory.store16(reg, uint16_t(
                int16_t(memory.load16(reg))
                >> memory.load8(arg1) & 15));
          }
        } else {
          if (imm_) { P_TRACE("shab 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, uint8_t(
                int8_t(memory.load8(reg)) >> arg1 & 7));
          } else { P_TRACE("shab 0x%X, (0x%X)", reg, arg1);
            memory.store8(reg, uint8_t(
                int8_t(memory.load8(reg)) >> memory.load8(arg1) & 7));
          }
        }
      } break;
      case BaseOp::Jl: {
        uint16_t label = memory.load16(ip + 4);
        if (bits16_) {
          if (imm_) { P_TRACE("jle 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (int16_t(memory.load16(reg)) <= int16_t(arg1)) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jle 0x%X, (0x%X), 0x%X", reg, arg1, label);
            if (
              int16_t(memory.load16(reg)) <= int16_t(memory.load16(arg1))
            ) {
              memory.store16(REG_IP, label);
            }
          }
        } else {
          if (imm_) { P_TRACE("jl 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (int16_t(memory.load16(reg)) < int16_t(arg1)) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jl 0x%X, (0x%X), 0x%X", reg, arg1, label);
            if (
              int16_t(memory.load16(reg)) < int16_t(memory.load16(arg1))
            ) {
              memory.store16(REG_IP, label);
            }
          }
        }
      } return;  // don't hit the "ip += 4"
      case BaseOp::Jg: {
        uint16_t label = memory.load16(ip + 4);
        if (bits16_) {
          if (imm_) { P_TRACE("jge 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (int16_t(memory.load16(reg)) >= int16_t(arg1)) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jge 0x%X, (0x%X), 0x%X", reg, arg1, label);
            if (
              int16_t(memory.load16(reg)) >= int16_t(memory.load16(arg1))
            ) {
              memory.store16(REG_IP, label);
            }
          }
        } else {
          if (imm_) { P_TRACE("jg 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (int16_t(memory.load16(reg)) > int16_t(arg1)) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jg 0x%X, (0x%X), 0x%X", reg, arg1, label);
            if (
              int16_t(memory.load16(reg)) > int16_t(memory.load16(arg1))
            ) {
              memory.store16(REG_IP, label);
            }
          }
        }
      } return;
      case BaseOp::Jb: {
        uint16_t label = memory.load16(ip + 4);
        if (bits16_) {
          if (imm_) { P_TRACE("jbe 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) <= arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jbe 0x%X, (0x%X), 0x%X", reg, arg1, label);
            if (memory.load16(reg) <= memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        } else {
          if (imm_) { P_TRACE("jb 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) < arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jb 0x%X, (0x%X), 0x%X", reg, arg1, label);
            if (memory.load16(reg) < memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        }
      } return;
      case BaseOp::Ja: {
        uint16_t label = memory.load16(ip + 4);
        if (bits16_) {
          if (imm_) { P_TRACE("jae 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) >= arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jae 0x%X, (0x%X), 0x%X", reg, arg1, label);
            if (memory.load16(reg) >= memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        } else {
          if (imm_) { P_TRACE("ja 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) > arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("ja 0x%X, (0x%X), 0x%X", reg, arg1, label);
            if (memory.load16(reg) > memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        }
      } return;
      case BaseOp::Jq: {
        uint16_t label = memory.load16(ip + 4);
        if (bits16_) {
          if (imm_) { P_TRACE("jnq 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) != arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jnq 0x%X, (0x%X), 0x%X", reg, arg1, label);
            if (memory.load16(reg) != memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        } else {
          if (imm_) { P_TRACE("jq 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) == arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jq 0x%X, (0x%X), 0x%X", reg, arg1, label);
            if (memory.load16(reg) == memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        }
      } return;
    }
    memory.store16(REG_IP, ip + 4);
  }
};


void interp_instr(Memory& memory) {
  auto op = Opcode(memory.load16(memory.load16(REG_IP)));
  op(memory);
}

int main(int argc, char **argv) {
  if (argc <= 1) {
    printf("Err: Specify file\n");
    return -1;
  }

  FILE* fd = fopen(argv[1], "r");
  if (!fd) {
    perror("File");
    return -1;
  }

#define MAX_CODE_SIZE 0xFFFF
  uint8_t code[MAX_CODE_SIZE] = {0};
  size_t code_size = fread(code, 1, MAX_CODE_SIZE, fd);
  if (ferror(fd)) {
    fprintf(stderr, "Failed to read file: %s", strerror(errno));
    std::terminate();
  }

  auto&& memory = Memory(code, code_size);

  setvbuf(stdout, NULL, _IONBF, 0); // unbuffer stdout

  while (1) {
    interp_instr(memory);
  }
}
