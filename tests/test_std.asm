ji start

import std.constants
import std.memeq
import std.printhex
import std.print

STRING: data "Hello, world!\n"
equ STRING_LEN 13

MESSAGE1: data " <- should be 0x0001\n"
equ MESSAGE1_LEN 21

MESSAGE2: data " <- should be 0x0000\n"
equ MESSAGE2_LEN 21

start:
	mi s00, STRING
	mi s01, STRING_LEN
	mi s02, STRING
	mi s03, STRING_LEN
	call memeq
	call printhex
	mi s00, MESSAGE1
	mi s01, MESSAGE1_LEN
	call print

	mi s00, STRING
	mi s01, STRING_LEN
	mi s02, MESSAGE1
	mi s03, MESSAGE1_LEN
	call memeq
	call printhex
	mi s00, MESSAGE2
	mi s01, MESSAGE2_LEN
	call print

	mi s00, MESSAGE1
	mi s01, MESSAGE1_LEN
	mi s02, MESSAGE2
	mi s03, MESSAGE2_LEN
	call memeq
	call printhex
	mi s00, MESSAGE2
	mi s01, MESSAGE2_LEN
	call print

	hf
