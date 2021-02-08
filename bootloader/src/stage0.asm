; Base address for first sector on x86 BIOS
[org 0x7c00]
; Contains 16-bit executable code
[bits 16]

entry:
    ; Disable interrupts cuz we dont have interrupt tables
    cli
    ; Makes the string instructions (stosb, movsb) go upward
    cld

    ; Set the A20 line so we can use > 1MB RAM
    in      al, 0x92
    or      al, 2
    out     0X92, al

    ; Clear DS
    xor     ax, ax
    mov     ds, ax

    ; Load a 32-bit GDT
    lgdt    [ds:pm_gdt]

    ; Enable protected mode
    mov     eax, cr0
    or      eax, (1 << 0)
    mov     cr0, eax

    ; Transition to 320bit mode by setting CS to a protected mode selector
    jmp     0x0008: pm_entry

[bits 32]

pm_entry:
    ; Set up all data selectors
    mov     ax, 0x10
    mov     es, ax
    mov     ds, ax
    mov     fs, ax
    mov     gs, ax
    mov     ss, ax

    cli
    hlt

; --------------------------------------------------------------------

; 32-bit protected mode GDT

align 8
pm_gdt_base:
    dq      0x0000000000000000
    dq      0x00CF9A000000FFFF
    dq      0x00CF92000000FFFF

pm_gdt:
    dw      (pm_gdt - pm_gdt_base) - 1
    dd      pm_gdt_base
