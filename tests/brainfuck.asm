JI start

; --- System constants ---
equ OUT 0x5
equ IN  0x6
equ SC2 0x4

; --- BF Constants ---
equ CURR_CMD 0xF0 ; ptr to currently executing cmd
equ DATA_POINTER 0xF1 ; ptr to current data

start:
    MI CURR_CMD, PROGRAM ; set our current command to PROGRAM
    MI DATA_POINTER, DATA_REGION ; Data pointer to beginning of data region

    DEC CURR_CMD ; back one so inc on first interp loop brings us to beginning
    JI interp

interp:
    INC CURR_CMD
    ;Debug: print current command
    ;  MI OUT, 0x43
    ;  MI OUT, 0x4d
    ;  MI OUT, 0x44
    ;  MI OUT, 0x3a
    ;  MI OUT, 0x20
    ;  MD OUT, CURR_CMD
	;  MI OUT, 0x0a
    MD SC2, CURR_CMD
    MI SC, 0x3e ; >
    JQ SC2, SC, inc_ptr
    MI SC, 0x3c ; <
    JQ SC2, SC, dec_ptr
    MI SC, 0x2b ; +
    JQ SC2, SC, inc_val
    MI SC, 0x2d ; -
    JQ SC2, SC, dec_val
    MI SC, 0x2e ; .
    JQ SC2, SC, put_val
    MI SC, 0x2c ; ,
    JQ SC2, SC, get_val
    MI SC, 0x5b ; [
    JQ SC2, SC, beg_loop
    MI SC, 0x5d ; ]
    JQ SC2, SC, end_loop
    MI SC, 0x0 ; NUL terminate str
    JQ SC2, SC, exit
    HF ; no matching instruction, fail

inc_ptr:
    INC DATA_POINTER
    JI interp

dec_ptr:
    DEC DATA_POINTER
    JI interp

inc_val:
    MD SC2, DATA_POINTER
    INC SC2
    ; MI OUT, 0x2d
    ; MV OUT, SC
    ; MI OUT, 0x2d
    ST SC2, DATA_POINTER
    JI interp
dec_val:
    MD SC2, DATA_POINTER
    DEC SC2
    ST SC2, DATA_POINTER
    JI interp

put_val:
    MD OUT, DATA_POINTER
    JI interp

get_val:
    LD DATA_POINTER, IN
    JI interp
    
beg_loop:
    MD SC2, DATA_POINTER
    MI SC, 0x0
    JQ SC2, SC, beg_loop__no_enter_loop
    JI beg_loop__enter_loop
beg_loop__no_enter_loop: ; we don't need to start looping -- data is 0
    ; eat until ']'
    MD SC2, CURR_CMD
    MI SC, 0x5d ; ']'
    JQ SC, SC2, beg_loop__after
    INC CURR_CMD
    JI beg_loop__no_enter_loop
beg_loop__after:
    JI interp
beg_loop__enter_loop:
    ; Push cmd ptr and enter loop
    PUSH CURR_CMD
    JI interp

end_loop:
    ; if *DP != 0, POP CURR_CMD, else POP SC
    MD SC2, DATA_POINTER
    MI SC, 0x0
    JQ SC2, SC, end_loop__no_loop
    ; fallthru to end_loop__do_loop
end_loop__do_loop:
    POP CURR_CMD
    DEC CURR_CMD ; back one
    JI interp
end_loop__no_loop:
    POP SC
    JI interp

exit:
    MI OUT, 0x42
    MI OUT, 0x79
    MI OUT, 0x65
    MI OUT, 0x21
    MI OUT, 0x0a
    HF


; --- BF Program to run ---
PROGRAM:
data "++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++\
			++++++++++.>.+++.------.--------.>+.>." 0x0

; currently 256 words -- rep macro would be useful to make this cleaner
DATA_REGION:
data rep 256 0x0
