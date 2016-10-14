#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <cstdlib>

#include "memory.h"

Memory mem;

#define INST_PTR_LOC 0x0
#define STK_PTR_LOC  0x1
#define BASE_PTR_LOC 0x2
#define SCRATCH_REG  0x3

#define CODE_START 0x1000

#define INSTR(opcode, value, family, doc) opcode = value,
enum op {
    INSTR(MV ,   0b0000, ARITH, "Move")
    INSTR(XG ,   0b0001, ARITH, "Exchange")
    INSTR(AD ,   0b0010, ARITH, "Add")
    INSTR(SB ,   0b0011, ARITH, "Subtract")
    INSTR(ND ,   0b0100, ARITH, "And (bitwise)")
    INSTR(OR ,   0b0101, ARITH, "Or (bitwise)")
    INSTR(XR ,   0b0110, ARITH, "Xor (bitwise)")
    INSTR(SR ,   0b0111, ARITH, "Shift Right")
    INSTR(SL ,   0b1000, ARITH, "Shift Left")
    INSTR(SA ,   0b1001, ARITH, "Arithmetic Shift Right")
    INSTR(MI ,   0b1010, MOV_IMM, "Move Immediate")
    INSTR(MD ,   0b1011, MOV_DER, "Move Dereference")
    INSTR(JG ,   0b1100, JUMP, "Jump if Greater-Than")
    INSTR(JL ,   0b1101, JUMP, "Jump if Less-Than")
    INSTR(JQ ,   0b1110, JUMP, "Jump if Equal-To")
    INSTR(HF1,   0b1111, HALT, "Halt and Catch Fire")
};

#define OPCODE(addr) (op((mem[addr] & 0b1111000000000000)>>12))

// Arith decoder macros
#define ARITH_LEN_BYTES 4
#define ARITH_DST(addr) ((mem[addr] & 0b0000111111111111))
#define ARITH_SRC(addr) ((mem[addr+1] & 0b1111111111111111))

// Move Imm decoder macros
#define MOVE_IMM_LEN_BYTES 4
#define MOVE_IMM_DST(addr) ((mem[addr] & 0b0000111111111111))
#define MOVE_IMM_IMM(addr) ((mem[addr+1] & 0b1111111111111111))

// Move Deref decoder macros
#define MOVE_DER_LEN_BYTES 4
#define MOVE_DER_DST(addr) ((mem[addr] & 0b0000111111111111))
#define MOVE_DER_SRC(addr) ((mem[addr+1] & 0b1111111111111111))

// Jump decoder macros
// Jumps are:
// if (lhs op rhs) ip = addr
#define JUMP_LEN_BYTES 6
#define JUMP_LHS(addr) ((mem[addr] & 0b0000111111111111))
#define JUMP_RHS(addr) ((mem[addr+1] & 0b1111111111111111))
#define JUMP_ADDR(addr) ((mem[addr+2] & 0b1111111111111111))


void interp_instr() {
    mem_t ip = mem[INST_PTR_LOC];
    mem_t next_ip = -1;
    enum op opcode = OPCODE(ip);
    mem_t dst, src, imm;
    mem_t lhs, rhs, addr;
    mem_t temp;
    switch (opcode) {
        case MV:
            next_ip = ip + (ARITH_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer

            dst = ARITH_DST(ip);
            src = ARITH_SRC(ip);
            printf("0x%04x: MV 0x%04x, 0x%04x\n", ip, dst, src);
            mem[dst] = mem[src];
            break;
        case XG:

            next_ip = ip + (ARITH_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer

            dst = ARITH_DST(ip);
            src = ARITH_SRC(ip);
            printf("0x%04x: XG 0x%04x, 0x%04x\n", ip, dst, src);
            // could swap with xor, but this is clear
            temp = mem[dst];
            mem[dst] = mem[src];
            mem[src] = temp;
            break;
        case AD:
            next_ip = ip + (ARITH_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer

            dst = ARITH_DST(ip);
            src = ARITH_SRC(ip);
            printf("0x%04x: AD 0x%04x, 0x%04x\n", ip, dst, src);
            mem[dst] = mem[dst] + mem[src];
            break;
        case SB:
            next_ip = ip + (ARITH_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            dst = ARITH_DST(ip);
            src = ARITH_SRC(ip);
            printf("0x%04x: SB 0x%04x, 0x%04x\n", ip, dst, src);
            mem[dst] = mem[dst] - mem[src];
            break;
        case ND:
            next_ip = ip + (ARITH_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            dst = ARITH_DST(ip);
            src = ARITH_SRC(ip);
            printf("0x%04x: ND 0x%04x, 0x%04x\n", ip, dst, src);
            mem[dst] = mem[dst] & mem[src];
            break;
        case OR:
            next_ip = ip + (ARITH_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            dst = ARITH_DST(ip);
            src = ARITH_SRC(ip);
            printf("0x%04x: OR 0x%04x, 0x%04x\n", ip, dst, src);
            mem[dst] = mem[dst] | mem[src];
            break;
        case XR:
            next_ip = ip + (ARITH_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            dst = ARITH_DST(ip);
            src = ARITH_SRC(ip);
            printf("0x%04x: XR 0x%04x, 0x%04x\n", ip, dst, src);
            mem[dst] = mem[dst] ^ mem[src];
            break;
        case SR:
            next_ip = ip + (ARITH_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            dst = ARITH_DST(ip);
            src = ARITH_SRC(ip);
            printf("0x%04x: SR 0x%04x, 0x%04x\n", ip, dst, src);
            mem[dst] = mem[dst] >> mem[src];
            break;
        case SL:
            next_ip = ip + (ARITH_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            dst = ARITH_DST(ip);
            src = ARITH_SRC(ip);
            printf("0x%04x: SL 0x%04x, 0x%04x\n", ip, dst, src);
            mem[dst] = mem[dst] << mem[src];
            break;
        case SA:
            next_ip = ip + (ARITH_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            dst = ARITH_DST(ip);
            src = ARITH_SRC(ip);
            printf("0x%04x: SA 0x%04x, 0x%04x\n", ip, dst, src);
            mem[dst] = (uint16_t) (((int16_t) mem[dst]) >> mem[src]);
            break;
        case MI:
            next_ip = ip + (MOVE_IMM_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            dst = MOVE_IMM_DST(ip);
            imm = MOVE_IMM_IMM(ip);
            printf("0x%04x: MI 0x%04x, 0x%04x\n", ip, dst, imm);
            mem[dst] = imm;
            break;
        case MD:
            next_ip = ip + (MOVE_DER_LEN_BYTES>>1);
            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            dst = MOVE_DER_DST(ip);
            src = MOVE_DER_SRC(ip);
            printf("0x%04x: MD 0x%04x, 0x%04x\n", ip, dst, imm);
            mem[dst] = mem[imm];
            break;
        case JG:
            lhs = JUMP_LHS(ip);
            rhs = JUMP_RHS(ip);
            addr = JUMP_ADDR(ip);
            printf("0x%04x: JG 0x%04x, 0x%04x, 0x%04x\n", ip, lhs, rhs, addr);

            if (lhs > rhs) {
                next_ip = addr;
            } else {
                next_ip = ip + (JUMP_LEN_BYTES>>1);
            }

            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            break;
        case JL:
            lhs = JUMP_LHS(ip);
            rhs = JUMP_RHS(ip);
            addr = JUMP_ADDR(ip);
            printf("0x%04x: JL 0x%04x, 0x%04x, 0x%04x\n", ip, lhs, rhs, addr);

            if (lhs < rhs) {
                next_ip = addr;
            } else {
                next_ip = ip + (JUMP_LEN_BYTES>>1);
            }

            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            break;
        case JQ:
            lhs = JUMP_LHS(ip);
            rhs = JUMP_RHS(ip);
            addr = JUMP_ADDR(ip);
            printf("0x%04x: JQ 0x%04x, 0x%04x, 0x%04x\n", ip, lhs, rhs, addr);

            if (lhs == rhs) {
                next_ip = addr;
            } else {
                next_ip = ip + (JUMP_LEN_BYTES>>1);
            }

            mem[INST_PTR_LOC] = next_ip; // update instruction pointer
            
            break;
        case HF1:
            printf("0x%04x: HF\n", ip);
            abort(); // TODO: Actually halt and catch fire
            
            break;
    }
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
    mem_t code[MAX_CODE_SIZE] = {0};
    size_t code_size = fread(code, 2, MAX_CODE_SIZE, fd);

    for (size_t i = 0; i < code_size; ++i) {
        mem[CODE_START + i] = code[i];
    }

    mem[INST_PTR_LOC] = CODE_START;

    while (1) {
        interp_instr();
    }
}
