JI start

equ SC2 0x4
equ OUT 0x200
equ IN  0x201

; argument locations for convenience
equ ARG1 0x40
equ ARG2 0x41
equ ARG3 0x42
equ ARG4 0x43

start:
MI ARG1, menu_entries
JI print_s ; TODO: call
HF


; print proc. location of first char in s is in ARG1. s is NUL terminated
print_s:
print_s__loop:
    MD SC2, ARG1
    MI SC, 0x0
    JQ SC, SC2, print_s__done
    ;fallthu to print_s__cont
print_s__cont:
    MV OUT, SC2 ; print char
    INC ARG1 ; advance to next character
    JI print_s__loop
print_s__done:
    HF ; TODO: RET

; ----- Strings
welcome_msg:
data "Welcome to the FBTTY Management Interface" 0x0a 0x00

menu_entries:
data "- MAKE SELECTION -"  0x0a \
"1) ASDF" 0x0a \
"2) POOP" 0x0a \
"3) WAT" 0x0a \
"4) LOL" 0x0a \
0x00

prompt:
data "SELECT: " 0x0
