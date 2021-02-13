import ctypes
import mmap

libc = ctypes.cdll.LoadLibrary(None)

# Getting libc mmap function and setting function signature
# void *mmap(void *addr, size_t length, int prot, int flags, int fd, off_t offset)
mmap_function = libc.mmap
mmap_function.restype = ctypes.c_void_p
mmap_function.argtypes = (ctypes.c_void_p, ctypes.c_size_t,
        ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_size_t)

CODE_SIZE = 1024

# Allocate RWX memory with mmap

code_address = mmap_function(None, CODE_SIZE, mmap.PROT_READ | mmap.PROT_WRITE | mmap.PROT_EXEC,
        mmap.MAP_PRIVATE | mmap.MAP_ANONYMOUS, -1, 0)

if code_address == -1:
    raise OSError('mmap failed to allocate memory')

# Move code to code_address
# mov %rdi, %rax
# add $4, %rax
# ret
code = b'\x48\x89\xf8\x48\x83\xc0\x04\xc3'
assert len(code) <= CODE_SIZE
ctypes.memmove(code_address, code, len(code))

JitFuncType = ctypes.CFUNCTYPE(ctypes.c_long, ctypes.c_long)
function = ctypes.cast(code_address, JitFuncType)
print(function(-100))
