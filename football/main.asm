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
	; jump to index entry in jumptable
    mi s00, jump_table
    ad s00, user__index
    md ip, s00
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
entry_0:  data jt0
entry_1:  data jt0
entry_2:  data jt0
entry_3:  data jt0
entry_4:  data jt0
entry_5:  data jt0
entry_6:  data jt0
entry_7:  data jt0
entry_8:  data jt0
entry_9:  data jt0
entry_10: data jt0
entry_11: data jt0
entry_12: data jt0
entry_13: data jt0
entry_14: data jt0

; a struct password (importantly, directly after jumptable)
password:
password__cksum: data 0
; password.data
user:
user__name: data rep 16 0
user__secret: data rep 8 0
user__index: data 0

; -- Jump targets
jt0:
    mi s00, tmp_junk_target_s
    mi s01, tmp_junk_target_s_len
    call println
    hf



tmp_junk_target_s:
data "DEFINE THIS JUMP TARGET HOMIE"

equ tmp_junk_target_s_len 29 
