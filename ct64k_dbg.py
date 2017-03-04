#!/usr/bin/env python3
from enum import Enum
from copy import deepcopy
import sys
import readline # make input() use readline <3

CODE_BASE_ADDR = 0x1000 # where code is loaded
MEMORY_SIZE = 0xFFFF # How big memory is

INSTR_PTR_LOC = 0x0
STACK_PTR_LOC = 0x1
BASE_PTR_LOC = 0x2

class InstructionFamily(Enum):
    IMM = 0
    ARTIH = 1
    JMP = 2


class Opcode(Enum):
    MI = 0x0
    MV = 0x1
    MD = 0x2
    LD = 0x3
    ST = 0x4
    AD = 0x5
    SB = 0x6
    ND = 0x7
    OR = 0x8
    XR = 0x9
    SR = 0xa
    SL = 0xb
    SA = 0xc
    JG = 0xd
    JL = 0xe
    JQ = 0xf

class Instruction:
    def __init__(self, address, opcode, *args):
        self.address = address
        self.opcode = Opcode(opcode)
        if self.opcode in [Opcode.MI]: 
            self.family = InstructionFamily.IMM
            self.rm = args[0]
            self.imm = args[1]
            self.length = 2 # num words
        elif self.opcode in [Opcode.MV, Opcode.MD, Opcode.LD, Opcode.ST, \
                             Opcode.AD, Opcode.SB, Opcode.ND, Opcode.OR, \
                             Opcode.XR, Opcode.SR, Opcode.SL, Opcode.SA]:
            self.family = InstructionFamily.ARTIH
            self.rm = args[0]
            self.mem = args[1]
            self.length = 2 # num words
        elif self.opcode in [Opcode.JG, Opcode.JL, Opcode.JQ]:
            self.family = InstructionFamily.JMP
            self.rm = args[0]
            self.mem = args[1]
            self.label = args[2]
            self.length = 3 # num words
        else:
            raise NotImplemented('WHAT EVEN THATS NOT A POSSIBLE OPCODE: {:x}'.format(self.opcode))
    
    def mnemonic(self):
        mnemonics = {
            Opcode.MI: 'MI',
            Opcode.MV: 'MV',
            Opcode.MD: 'MD',
            Opcode.LD: 'LD',
            Opcode.ST: 'ST',
            Opcode.AD: 'AD',
            Opcode.SB: 'SB',
            Opcode.ND: 'ND',
            Opcode.OR: 'OR',
            Opcode.XR: 'XR',
            Opcode.SR: 'SR',
            Opcode.SL: 'SL',
            Opcode.SA: 'SA',
            Opcode.JG: 'JG',
            Opcode.JL: 'JL',
            Opcode.JQ: 'JQ'
        }
        mnemonic = mnemonics[self.opcode]
        if mnemonic == 'JQ' and self.rm == 0x0 and self.mem == 0x0 and self.address == self.label:
            mnemonic = 'HF' # special case for halt-and-catch-fire, which is a specialization of jmp to self
        return mnemonic

    def __repr__(self):
        if self.family == InstructionFamily.IMM:
            return 'Instruction(0x{:x}, 0x{:x}, [0x{:x}, 0x{:x}])'.format( \
                    self.address, self.opcode, \
                    self.rm, self.imm)
        elif self.family == InstructionFamily.ARTIH:
            return 'Instruction(0x{:x}, 0x{:x}, [0x{:x}, 0x{:x}])'.format( \
                    self.address, self.opcode, \
                    self.rm, self.mem)
        elif self.family == InstructionFamily.JMP:
            return 'Instruction(0x{:x}, 0x{:x}, [0x{:x}, 0x{:x}, 0x{:x}])'.format( \
                    self.address, self.opcode, \
                    self.rm, self.mem, self.label)
        else:
            raise NotImplemented('What even instruction did you give me')

    def __str__(self):
        """
        If you want a prettier version, use .nice_str
        """
        if self.family == InstructionFamily.IMM:
            return '{} 0x{:x}, 0x{:x}'.format(self.mnemonic(), self.rm, self.imm)
        elif self.family == InstructionFamily.ARTIH:
            return '{} 0x{:x}, 0x{:x}'.format(self.mnemonic(), self.rm, self.mem)
        elif self.family == InstructionFamily.JMP:
            if self.opcode == Opcode.JQ and self.rm == 0x0 and self.mem == 0x0 and self.address == self.label:
                return 'HF' # special case for halt-and-catch-fire
            return '{} 0x{:x}, 0x{:x}, 0x{:x}'.format(self.mnemonic(), self.rm, self.mem, self.label)

    def nice_str(self, symbol_fn):
        """
        symbol_fn takes an address and returns a pretty string for it.
        """
        if self.family == InstructionFamily.IMM:
            return '{} {}, {}'.format(self.mnemonic(), symbol_fn(self.rm), symbol_fn(self.imm))
        elif self.family == InstructionFamily.ARTIH:
            return '{} {}, {}'.format(self.mnemonic(), symbol_fn(self.rm), symbol_fn(self.mem))
        elif self.family == InstructionFamily.JMP:
            if self.opcode == Opcode.JQ and self.rm == 0x0 and self.mem == 0x0 and self.address == self.label:
                return 'HF' # special case for halt-and-catch-fire
            return '{} {}, {}, {}'.format(self.mnemonic(), symbol_fn(self.rm), symbol_fn(self.mem), symbol_fn(self.label))
    
    @classmethod
    def decode(cls, address, instruction_memory):
        opcode = (instruction_memory[0] >> 12) & 0xF
        args = []
        args.append(instruction_memory[0] & 0xFFF)
        args.append(instruction_memory[1])
        args.append(instruction_memory[2])
        return cls(address, opcode, *args)


class InvalidAccess(Exception):
    """
    Quick helper for invalid accesses
    """
    pass


def invalid_access(state, addr, *args):
    """
    Interrupt to represent things as invalid
    """
    raise InvalidAccess('Cannot read/write memory at addr {:x}'.format(addr))


def getchar_interrupt(state, addr, *args):
    """
    getchar interrupt
    """
    return state.get_input()

def putchar_interrupt(state, addr, *args):
    """
    putchar interrupt
    """
    char = args[0]
    state.add_output(chr(char))


class BreakpointHit(Exception):
    """
    Raise when a bp is hit
    """
    pass


class Memory:
    def __init__(self, memory):
        self.memory = memory
        self.state = None # fill this in l8r

    def __getitem__(self, key):
        if not isinstance(key, slice) and \
                key in self.state.read_interrupts:
            return self.state.read_interrupts[key](self.state, key)
        return self.memory[key]
    
    def __setitem__(self, key, value):
        if key in self.state.write_interrupts:
            self.state.write_interrupts[key](self.state, key, value)
            return
        self.memory[key] = value

class State:
    def __init__(self, memory):
        self.memory = memory
        self.halt = False # is cpu halted
        # addr read -> function to call for value to be read
        self.read_interrupts = {
            0x200: invalid_access, # reading output register
            0x201: getchar_interrupt, # getchar
            0x202: invalid_access,  # gen key write
            0x203: None,  # gen key read
            0x204: invalid_access,  # load key write
            0x205: None,  # load key read
            0x206: invalid_access,  # mul write
            0x207: None,  # mul read
            0x208: None,  # rng
            0x209: invalid_access,  # set timeout
            0x20A: None,  # timeout addr
        }
        # addr written -> function to call with writen value
        self.write_interrupts = {
            0x200: putchar_interrupt, # putchar
            0x201: invalid_access, # writing input register
            0x202: None,  # gen key write
            0x203: invalid_access,  # gen key read
            0x204: None,  # load key write
            0x205: invalid_access,  # load key read
            0x206: None,  # mul write
            0x207: invalid_access,  # mul read
            0x208: invalid_access,  # rng
            0x209: None,  # set timeout
            0x20A: None,  # timeout addr
        }
        self.all_input = ''
        self.all_output = ''

    def decode_instruction_at(self, addr):
        instr_memory = self.memory[addr : addr + 3] # we need 3 words for jmp family instructions
        return Instruction.decode(addr, instr_memory)

    def instruction_pointer(self):
        return self.memory[INSTR_PTR_LOC]

    def stack_pointer(self):
        return self.memory[STACK_PTR_LOC]

    def step(self):
        ip = self.instruction_pointer()
        instr = self.decode_instruction_at(ip)
        next_ip = self.instruction_pointer() + instr.length
        self.memory[INSTR_PTR_LOC] = next_ip # advance ip
        if instr.opcode == Opcode.MI:
            self.memory[instr.rm] = instr.imm
        elif instr.opcode == Opcode.MV:
            self.memory[instr.rm] = self.memory[instr.mem]
        elif instr.opcode == Opcode.MD:
            self.memory[instr.rm] = self.memory[self.memory[instr.mem]]
        elif instr.opcode == Opcode.LD:
            self.memory[self.memory[instr.rm]] = self.memory[instr.mem]
        elif instr.opcode == Opcode.ST:
            self.memory[self.memory[instr.mem]] = self.memory[instr.rm]
        elif instr.opcode == Opcode.AD:
            self.memory[instr.rm] += self.memory[instr.mem]
        elif instr.opcode == Opcode.SB:
            self.memory[instr.rm] -= self.memory[instr.mem]
        elif instr.opcode == Opcode.ND:
            self.memory[instr.rm] &= self.memory[instr.mem]
        elif instr.opcode == Opcode.OR:
            self.memory[instr.rm] |= self.memory[instr.mem]
        elif instr.opcode == Opcode.XR:
            self.memory[instr.rm] ^= self.memory[instr.mem]
        elif instr.opcode == Opcode.SR:
            self.memory[instr.rm] >>= self.memory[instr.mem]
        elif instr.opcode == Opcode.SL:
            self.memory[instr.rm] <<= self.memory[instr.mem]
        elif instr.opcode == Opcode.SA:
            rm = self.memory[instr.rm]
            mem = self.memory[instr.mem]
            to_signed = lambda x: int.from_bytes(x.to_bytes(2, 'little'), 'little', True)
            to_unsigned = lambda x: int.from_bytes(x.to_bytes(2, 'little', True), 'little')
            rm = to_signed(rm)
            res = rm >> mem
            memory[rm] = to_unsigned(rm)
        elif instr.opcode == Opcode.JG:
            if self.memory[instr.rm] > self.memory[instr.mem]:
                self.memory[INSTR_PTR_LOC] = instr.label
        elif instr.opcode == Opcode.JL:
            if self.memory[instr.rm] < self.memory[instr.mem]:
                self.memory[INSTR_PTR_LOC] = instr.label
        elif instr.opcode == Opcode.JQ:
            if instr.rm == 0 and instr.mem == 0 and instr.label == ip:
                # halt-and-catch-fire
                self.cpu_abort()
            elif self.memory[instr.rm] == self.memory[instr.mem]:
                self.memory[INSTR_PTR_LOC] = instr.label

    def run(self):
        while not self.halt:
            instr = self.decode_instruction_at(self.instruction_pointer())
            self.step()

    def cpu_abort(self):
        self.halt = True

    def get_input(self):
        char = sys.stdin.read(1)
        self.all_input += char
        return ord(char)

    def add_output(self, value):
        self.all_output += value
        sys.stdout.write(value)
        sys.stdout.flush()

    @classmethod
    def from_file(cls, filename):
        with open(filename, 'rb') as f:
            contents = f.read()

        # reinterpret contents as an array of little-endian words
        code = []
        for i in range(0, len(contents), 2):
            mem = contents[i : i+2]
            word = int.from_bytes(mem, 'little')
            code.append(word)

        # memory space, all 0's to start
        memory = [0 for _ in range(MEMORY_SIZE+1)]

        # initialize stack ptr
        memory[STACK_PTR_LOC] = 0x300
        # initialize instruction ptr
        memory[INSTR_PTR_LOC] = 0x1000

        # initialize code
        for offset, word in enumerate(code):
            memory[CODE_BASE_ADDR + offset] = word

        st = State(Memory(memory))
        st.memory.state = st
        return st

class InteractiveDebugger:
    """
    An interactive debugger.

    Commands:
    again: repeat last command (just hitting return also does this)
    r: run, start the program if not running
    s, ni, n: single step.
        - Takes argument of number of instructions to step (default 1)
    c: run until breakpoint.
    b: break at address.
        - for example, b 0x1234 creates a breakpoint at the instruction 0x1234
    p: print expression. Takes a format specifier, defaulting to "word"
        - Format specifiers come in the form p/NX, 
          where N is a number of consecutive version of the argument type to print, 
          and X is the specifier.
          supports the following specifiers:
            w, x: word
            c: character
            i: instruction
    load-labels: Load labels from a file (argument required)

    Expressions: **TODO**
    right now just hex numbers for an addr haha
    """
    def __init__(self, state):
        self.initial_state = state
        self.state = None
        self.running = False

        self.symbols = {
            INSTR_PTR_LOC: 'IP',
            STACK_PTR_LOC: 'SP',
            0x3: 'SC0',
            0x4: 'SC1',
            0x200: 'OUTPUT',
            0x201: 'INPUT',
        }
        # rNN registers
        for i, x in enumerate(range(0x10, 0x40)):
            name = 'r{:02x}'.format(i)
            self.symbols[x] = name

        # sNN registers
        for i, x in enumerate(range(0x40, 0x100)):
            name = 's{:02x}'.format(i)
            self.symbols[x] = name
        
        self.breakpoints = {} # bp number -> bp addr
        self.highest_breakpoint_num = 0 # breakpoint #s monotonically increment
    
        self.last_command = ''

        # commands to run always
        self.autocmds = [
            #'p/10i *$ip', # instruction leadup
        ]

        self.watch_addrs = [INSTR_PTR_LOC, STACK_PTR_LOC]

    def input_loop(self):
        while True:
            self.print_banner()
            self.handle_input()

    def handle_input(self):
        """
        Handle getting input, parsing it, executing commands
        """
        inp = input('ctdbg> ')
        self.runcmd(inp)

    def runcmd(self, cmd, user=True):
        def print_stuff(num, specifier, loc):
            specifiers = {
                'x': lambda addr: hex(self.state.memory[addr]),
                'c': lambda addr: bytes([self.state.memory[addr]]),
                'i': lambda addr: hex(addr) + ': ' + self.state.decode_instruction_at(addr).nice_str(self.nice_format_addr),
            }
            specifiers['w'] = specifiers['x'] # w = x

            joinchrs = {
                'x': ' ',
                'c': b'',
                'i': '\n',
            }
            joinchrs['w'] = joinchrs['x'] # ditto to above

            try:
                fmt_fn = specifiers[specifier]
                # gather all outs into this nice lil array
                outs = []
                offset = 0
                for i in range(num):
                    addr = loc + offset
                    if specifier != 'i':
                        offset += 1
                    else:
                        offset += self.state.decode_instruction_at(addr).length
                    outs.append((addr, fmt_fn(addr)))
                #TODO: decide ways of formatting based on length
                out = joinchrs[specifier] + joinchrs[specifier].join(y for x,y in outs)
                if specifier == 'c': # get a repr for non-printables :)
                    out = repr(out)
                return out
            except Exception as e:
                print("err> Error formatting:", e)
        
        def step(n):
            if not self.running:
                self.restart()

            if self.state.halt:
                print('err> STATE HALTED, USE r TO RESTART')
                return

            for _ in range(n):
                self.state.step()

        def run():
            self.restart()
            self.run()

        def add_watch(addr):
            self.watch_addrs.append(addr)
            return 'Added a watchpoint for {:x}'.format(addr)


        cmd_dict = {
            'again': lambda: self.runcmd(self.last_command),
            'r': lambda: run(), # different then self.run
            's': lambda n: step(n),
            'c': lambda: self.run(),
            'b': lambda target: self.add_breakpoint(target),
            'p': lambda num, specifier, loc: print_stuff(num, specifier, loc),
            'd': lambda num: self.del_breakpoint(num),
            'watch': add_watch,
            'load-labels': lambda filename: self.load_symbols_from_file(filename),
        }

        try:
            parsed = self.parse_command(cmd)
            cmd_fn = cmd_dict[parsed[0]]
            args = parsed[1:]
            res = cmd_fn(*args)

            if res:
                print('out>', res)

            # only set last_command if user-initiated and not a repeat command
            if user and cmd not in {'again', ''}:
                self.last_command = cmd
        except ValueError as e:
            print('err> Unknown command:', cmd)
            print(e)

    def run(self):
        if not self.running:
            self.restart()

        try:
            while not self.state.halt:
                self.state.step()
                # bp check
                if self.state.instruction_pointer() in self.breakpoints.values():
                    raise BreakpointHit()
        except BreakpointHit:
            print('out> breakpoint hit!!')
        except KeyboardInterrupt:
            print('out> paused!!')

        if self.state.halt:
            self.running = False
            print('out> program hlt!!')

    def nice_format_addr(self, addr):
        """
        Format an address in the nicest way we can figure out
        """
        if addr in self.symbols:
            return '{} (0x{:x})'.format(self.symbols[addr], addr)
        
        return hex(addr)

    def print_banner(self):
        """
        Print a banner of watched variables, plus some other info
        (if cpu is halted, upcoming instructions, etc)
        """
        print('<===================')
        if self.state:
            for addr in self.watch_addrs:
                nice_addr, val = self.nice_format_addr(addr), self.state.memory[addr]
                print('{} = {:x}'.format(nice_addr, val))

            if self.state.halt:
                print('CPU HALTED')

            print('\n====================\n')

            for cmd in self.autocmds:
                try:
                    self.runcmd(cmd, user=False)
                except Exception as e:
                    pass
        else:
            print('Not running!')
        print('===================>')
        
    def parse_command(self, cmd):
        operation = cmd.split(' ')[0]
        arg = ' '.join(cmd.split(' ')[1:])
        if operation == '' or operation == 'again':
            return ('again',) # do last command again
        elif operation == 'r':
            return ('r',)
        elif operation in {'s', 'ni', 'n'}:
            if arg:
                arg = self.parse_expression(arg)
            else:
                arg = 1
            return ('s', 1)
        elif operation == 'c':
            return ('c',)
        elif operation == 'b':
            if not arg:
                raise ValueError('Argument required for b command')
            return ('b', self.parse_expression(arg))
        elif operation == 'watch':
            if not arg:
                raise ValueError('Argument required for watch command')
            return ('watch', self.parse_expression(arg))
        elif operation[0] in {'p', 'x'}:
            if '/' in operation:
                p_len = operation[operation.index('/') + 1 : -1]
                if p_len:
                    p_len = int(p_len)
                else:
                    p_len = 1
                p_specifier = operation[-1]
            else:
                p_len = 1
                p_specifier = 'w'
            return ('p', p_len, p_specifier, self.parse_expression(arg))
        elif operation == 'load-labels':
            if arg:
                return ('load-labels', arg)
            raise ValueError('Argument required for load-labels command')
        else:
            raise ValueError('Unknown command!')

    def parse_expression(self, expression):
        #TODO: * for deref, + and -
        # TODO: better gooder parsing?
        expression = expression.lower()
        for addr, name in self.symbols.items():
            expression = expression.replace('$' + name.lower(), hex(addr))
        needs_deref = False
        if expression[0] == '*':
            expression = expression[1:]
            needs_deref = True
        val = int(expression, 16)
        if needs_deref:
            val = self.state.memory[val]
        return val

    def add_breakpoint(self, addr):
        max_bp = self.highest_breakpoint_num
        bp_num = max_bp + 1
        self.breakpoints[bp_num] = addr
        self.highest_breakpoint_num += 1
        return bp_num

    def del_breakpoint(self, num):
        self.breakpoints.pop(num)
    
    def restart(self):
        """
        Restart the program under test (basically the `r` command)
        """
        self.state = deepcopy(self.initial_state)
        self.running = True

    def load_symbols_from_file(self, filename):
        new_symbols = {}
        with open(filename, 'r') as f:
            for line in f:
                if line.strip():
                    addr, name = line.split(':')
                    addr = int(addr, 0)
                    name = name.strip()
                    new_symbols[addr] = name
        print('Loaded {} symbols'.format(len(new_symbols)))
        self.symbols.update(new_symbols)


def main(argv):
    if len(argv) != 2:
        print('ERROR: ROM argument required!')
        return

    st = State.from_file(argv[1])
    dbg = InteractiveDebugger(st)
    dbg.runcmd('load-labels symbols')
    dbg.input_loop()


if __name__ == '__main__':
    main(sys.argv)
