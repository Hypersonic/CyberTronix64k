import std.constants
import std.print

ji start

STRING: data "Hello, world!\n"
equ STRING_LEN 13

start:
	mi s00, STRING
	mi s01, STRING_LEN
	call print
	hf
