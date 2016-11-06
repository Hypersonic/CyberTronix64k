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
#  define P_TRACE(fmt, ...) fprintf(stderr, "%04x: " fmt "\n", ip, __VA_ARGS__)
#else
#  define P_TRACE(fmt, ...)
#endif


class Opcode {
  enum class BaseOp: uint8_t {
    Mi = 0x0,
    Mm = 0x1,
    Md = 0x2,
    Nd = 0x3,
    Or = 0x4,
    Xr = 0x5,
    Ad = 0x6,
    Sb = 0x7,
    Sr = 0x8,
    Sl = 0x9,
    Sa = 0xA,
    Jl = 0xB,
    Jg = 0xC,
    Jb = 0xD,
    Ja = 0xE,
    Jq = 0xF
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
    auto arg1 = memory.load16(ip + 2);

    memory.store16(REG_IP, ip + 4);
    switch (base_) {
      case BaseOp::Mi: {
        if (bits16_) {
          if (!imm_) { P_TRACE("mi 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, arg1);
          } else { P_TRACE("li 0x%X, 0x%X", reg, arg1);
            memory.store16(memory.load16(reg), arg1);
          }
        } else {
          if (!imm_) {
            if (reg == 0 && arg1 == 0) {
              P_TRACE("mib 0x%X, 0x%X (hcf)", reg, arg1);
              printf("HCF instruction reached\n");
              std::exit(0);
            }
            P_TRACE("mib 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, uint8_t(arg1));
          } else { P_TRACE("lib 0x%X, 0x%X", reg, arg1);
            memory.store8(memory.load16(reg), uint8_t(arg1));
          }
        }
      } break;
      case BaseOp::Mm: {
        if (bits16_) {
          if (!imm_) { P_TRACE("mm 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(arg1));
          } else { P_TRACE("lm 0x%X, 0x%X", reg, arg1);
            memory.store16(memory.load16(reg), memory.load16(arg1));
          }
        } else {
          if (!imm_) { P_TRACE("mmb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(arg1));
          } else { P_TRACE("lmb 0x%X, 0x%X", reg, arg1);
            memory.store8(memory.load16(reg), memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Md: {
        if (bits16_) {
          if (!imm_) { P_TRACE("md 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(memory.load16(arg1)));
          } else { P_TRACE("ld 0x%X, 0x%X", reg, arg1);
            memory.store16(memory.load16(reg),
              memory.load16(memory.load16(arg1)));
          }
        } else {
          if (!imm_) { P_TRACE("mdb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(memory.load16(arg1)));
          } else { P_TRACE("ldb 0x%X, 0x%X", reg, arg1);
            memory.store8(memory.load16(reg),
              memory.load8(memory.load16(arg1)));
          }
        }
      } break;
      case BaseOp::Nd: {
        if (bits16_) {
          if (imm_) { P_TRACE("ndi 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) & arg1);
          } else { P_TRACE("nd 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) & memory.load16(arg1));
          }
        } else {
          if (imm_) { P_TRACE("ndbi 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) & uint8_t(arg1));
          } else { P_TRACE("ndb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) & memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Or: {
        if (bits16_) {
          if (imm_) { P_TRACE("ori 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) | arg1);
          } else { P_TRACE("or 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) | memory.load16(arg1));
          }
        } else {
          if (imm_) { P_TRACE("orbi 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) | uint8_t(arg1));
          } else { P_TRACE("orb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) | memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Xr: {
        if (bits16_) {
          if (imm_) { P_TRACE("xri 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) ^ arg1);
          } else { P_TRACE("xr 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) ^ memory.load16(arg1));
          }
        } else {
          if (imm_) { P_TRACE("xrbi 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) ^ uint8_t(arg1));
          } else { P_TRACE("xrb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) ^ memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Ad: {
        if (bits16_) {
          if (imm_) { P_TRACE("adi 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) + arg1);
          } else { P_TRACE("ad 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) + memory.load16(arg1));
          }
        } else {
          if (imm_) { P_TRACE("adb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) + uint8_t(arg1));
          } else { P_TRACE("adbi 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) + memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Sb: {
        if (bits16_) {
          if (imm_) { P_TRACE("sbi 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) - arg1);
          } else { P_TRACE("sb 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) - memory.load16(arg1));
          }
        } else {
          if (imm_) { P_TRACE("sbbi 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) - uint8_t(arg1));
          } else { P_TRACE("sbb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) - memory.load8(arg1));
          }
        }
      } break;
      case BaseOp::Sr: {
        if (bits16_) {
          if (imm_) { P_TRACE("sri 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) >> arg1 & 15);
          } else { P_TRACE("sr 0x%X, 0x%X", reg, arg1);
            memory.store16(reg,
              memory.load16(reg) >> memory.load8(arg1) & 15);
          }
        } else {
          if (imm_) { P_TRACE("srbi 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) >> arg1 & 7);
          } else { P_TRACE("srb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg,
              memory.load8(reg) >> memory.load8(arg1) & 7);
          }
        }
      } break;
      case BaseOp::Sl: {
        if (bits16_) {
          if (imm_) { P_TRACE("sli 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, memory.load16(reg) << arg1 & 15);
          } else { P_TRACE("sl 0x%X, 0x%X", reg, arg1);
            memory.store16(reg,
              memory.load16(reg) << memory.load8(arg1) & 15);
          }
        } else {
          if (imm_) { P_TRACE("slbi 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, memory.load8(reg) << arg1 & 7);
          } else { P_TRACE("slb 0x%X, 0x%X", reg, arg1);
            memory.store8(reg,
              memory.load8(reg) << memory.load8(arg1) & 7);
          }
        }
      } break;
      case BaseOp::Sa: {
        if (bits16_) {
          if (imm_) { P_TRACE("sai 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, uint16_t(
                int16_t(memory.load16(reg)) >> arg1 & 15));
          } else { P_TRACE("sa 0x%X, 0x%X", reg, arg1);
            memory.store16(reg, uint16_t(
                int16_t(memory.load16(reg))
                >> memory.load8(arg1) & 15));
          }
        } else {
          if (imm_) { P_TRACE("sabi 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, uint8_t(
                int8_t(memory.load8(reg)) >> arg1 & 7));
          } else { P_TRACE("sab 0x%X, 0x%X", reg, arg1);
            memory.store8(reg, uint8_t(
                int8_t(memory.load8(reg)) >> memory.load8(arg1) & 7));
          }
        }
      } break;
      case BaseOp::Jl: {
        auto label = memory.load16(ip + 4); ip += 2;
        if (bits16_) {
          if (imm_) { P_TRACE("jlei 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (int16_t(memory.load16(reg)) <= int16_t(arg1)) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jle 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (
              int16_t(memory.load16(reg)) <= int16_t(memory.load16(arg1))
            ) {
              memory.store16(REG_IP, label);
            }
          }
        } else {
          if (imm_) { P_TRACE("jli 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (int16_t(memory.load16(reg)) < int16_t(arg1)) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jl 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (
              int16_t(memory.load16(reg)) < int16_t(memory.load16(arg1))
            ) {
              memory.store16(REG_IP, label);
            }
          }
        }
      } break;
      case BaseOp::Jg: {
        auto label = memory.load16(ip + 4); ip += 2;
        if (bits16_) {
          if (imm_) { P_TRACE("jgei 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (int16_t(memory.load16(reg)) >= int16_t(arg1)) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jge 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (
              int16_t(memory.load16(reg)) >= int16_t(memory.load16(arg1))
            ) {
              memory.store16(REG_IP, label);
            }
          }
        } else {
          if (imm_) { P_TRACE("jgi 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (int16_t(memory.load16(reg)) > int16_t(arg1)) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jg 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (
              int16_t(memory.load16(reg)) > int16_t(memory.load16(arg1))
            ) {
              memory.store16(REG_IP, label);
            }
          }
        }
      } return;
      case BaseOp::Jb: {
        auto label = memory.load16(ip + 4); ip += 2;
        if (bits16_) {
          if (imm_) { P_TRACE("jbei 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) <= arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jbe 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) <= memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        } else {
          if (imm_) { P_TRACE("jbi 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) < arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jb 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) < memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        }
      } return;
      case BaseOp::Ja: {
        auto label = memory.load16(ip + 4); ip += 2;
        if (bits16_) {
          if (imm_) { P_TRACE("jaei 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) >= arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jae 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) >= memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        } else {
          if (imm_) { P_TRACE("jai 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) > arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("ja 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) > memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        }
      } return;
      case BaseOp::Jq: {
        auto label = memory.load16(ip + 4); ip += 2;
        if (bits16_) {
          if (imm_) { P_TRACE("jnqi 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) != arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jnq 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) != memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        } else {
          if (imm_) { P_TRACE("jqi 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) == arg1) {
              memory.store16(REG_IP, label);
            }
          } else { P_TRACE("jq 0x%X, 0x%X, 0x%X", reg, arg1, label);
            if (memory.load16(reg) == memory.load16(arg1)) {
              memory.store16(REG_IP, label);
            }
          }
        }
      } return;
    }
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
