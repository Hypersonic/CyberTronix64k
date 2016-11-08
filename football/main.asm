ji start
import std.constants
import std.getline
import std.println
import std.printhex
import std.base64
import std.memeq

import user
import cksum
import swizzle


start:
	call get_input
	hf

get_input:	
	mi s00, password_entry_s
	mi s01, password_entry_s_len
	call print
	mi s00, input_string
	mi s01, 256
	call getline
	st s01, input_string_len_p
	mi sc0, 78 ; max length of encoded data
	jq sc0, s01, good_len
err_bad_len:
	mi s00, bad_len_s
	mi s01, bad_len_s_len
	call println
	hf
good_len:
	mi s00, password
	mi s01, input_string
	mi s02, 26
    call base64dec
    ; unswizzle
    mi s00, user
    call swizzle
    call validate_cksum
    ; check valid idxes
    mi sc0, 0b10000
    jl sc0, user__index, err_bad_index
good_index:
	; TODO: jump to index entry
	ret

err_bad_index:
    mi s00, bad_idx_s
    mi s01, bad_idx_s_len
    call println
    hf
    

; returns if user cksum and password cksum match
validate_cksum:
    mi s00, user
    mi s01, 25
    call cksum
    jq s00, password__cksum, good_cksum
    mi s00, bad_cksum_s
    mi s01, bad_cksum_s_len
    call println
    hf
good_cksum:
    ret

password_entry_s:
data "PASSWORD: "

equ password_entry_s_len 10

bad_len_s:
data "BAD LENGTH, TERMINATING"

equ bad_len_s_len 23

bad_cksum_s:
data "BAD CKSUM, TERMINATING"

equ bad_cksum_s_len 22

bad_idx_s:
data "BAD INDEX, TERMINATING"

equ bad_idx_s_len 22

input_string:
data rep 256 0

input_string_len:
data 0

; pointer for stores
input_string_len_p:
data input_string_len

; jump table, length 15
jump_table:
entry_0:  data 0x0000
entry_1:  data 0x0000
entry_2:  data 0x0000
entry_3:  data 0x0000
entry_4:  data 0x0000
entry_5:  data 0x0000
entry_6:  data 0x0000
entry_7:  data 0x0000
entry_8:  data 0x0000
entry_9:  data 0x0000
entry_10: data 0x0000
entry_11: data 0x0000
entry_12: data 0x0000
entry_13: data 0x0000
entry_14: data 0x0000

; a struct password (importantly, directly after jumptable)
password:
password__cksum: data 0
; password.data
user:
user__name: data rep 16 0
user__secret: data rep 8 0
user__index: data 0
