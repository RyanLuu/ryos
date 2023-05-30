# import linker symbols
.section .rodata

.global TEXT_START
TEXT_START: .dword __text_start

.global TEXT_END
TEXT_END: .dword __text_end

.global RODATA_START
RODATA_START: .dword __rodata_start

.global RODATA_END
RODATA_END: .dword __rodata_end

.global DATA_START
DATA_START: .dword __data_start

.global DATA_END
DATA_END: .dword __data_end

.global BSS_START
BSS_START: .dword __bss_start

.global BSS_END
BSS_END: .dword __bss_end

.global STACK_START
STACK_START: .dword __stack_start

.global STACK_END
STACK_END: .dword __stack_end

.global HEAP_START
HEAP_START: .dword __heap_start

.global HEAP_END
HEAP_END: .dword __heap_end

