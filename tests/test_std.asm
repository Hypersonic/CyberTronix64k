ji start

import std.constants
import std.memcpy
import std.print

MESSAGE1: data "Blerghle\n"
equ MESSAGE1_LEN 9

MESSAGE2: data rep 9 0x0
equ MESSAGE2_LEN 9

start:
	mi s00, MESSAGE2
	mi s01, MESSAGE1
	mi s02, MESSAGE1_LEN
	call memcpy
	call print

	hf
