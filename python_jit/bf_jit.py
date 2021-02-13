import peachpy
from peachpy.x86_64 import *
import ctypes
import sys
from collections import namedtuple

def die(msg):
    print('ERROR:', msg)
    sys.exit(1)

def parse_bf_program(path):
    with open(path) as f:
        data = f.read()
    instructions = []
    for c in data:
        if c in ['<', '>', '+', '-', '.', ',', '[', ']']:
            instructions.append(c)
    return instructions

BracketLabels = namedtuple('BracketLabels', ('open_label', 'close_label'))

def jit_compile(bf_path):
    memptr = peachpy.Argument(peachpy.ptr(peachpy.uint8_t))

    open_brackets_stack = []
    with Function("ppjit",
                    [memptr],
                    result_type=None) as asm_function:
        dataptr = r13
        LOAD.ARGUMENT(dataptr, memptr)
        for instr in parse_bf_program(bf_path):
            if instr == '>':
                ADD(dataptr, 1)
            elif instr == '<':
                SUB(dataptr, 1)
            elif instr == '+':
                ADD([dataptr], 1)
            elif instr == '-':
                SUB([dataptr], 1)
            elif instr == '.':
                if sys.platform == "darwin":
                    MOV(rax, 0x2000004)
                else:
                    MOV(rax, 1)
                MOV(rdi, 1)
                MOV(rsi, dataptr)
                MOV(rdx, 1)
                SYSCALL()
            elif instr == ',':
                if sys.platform == "darwin":
                    MOV(rax, 0x2000003)
                else:
                    MOV(rax, 0)
                MOV(rdi, 0)
                MOV(rsi, dataptr)
                MOV(rdx, 1)
                SYSCALL()
            elif instr == '[':
                loop_start_label = Label()
                loop_end_label = Label()
                CMP([dataptr], 0)
                JZ(loop_end_label)
                LABEL(loop_start_label)
                open_brackets_stack.append(BracketLabels(loop_start_label, loop_end_label))
            elif instr == ']':
                if not len(open_brackets_stack):
                    die("no matching bracket")

                labels = open_brackets_stack.pop()

                CMP([dataptr], 0)
                JNZ(labels.open_label)

                LABEL(labels.close_label)

        RETURN()



    abi = peachpy.x86_64.abi.detect()
    encoded_function = asm_function.finalize(abi).encode()
    python_function = encoded_function.load()

    memsize = 30000
    MemoryArrayType = ctypes.c_uint8 * memsize
    memory = MemoryArrayType(*([0]*memsize))

    python_function(memory)
print(sys.platform)
sys.setrecursionlimit(10000)
jit_compile("./mandel.bf")
