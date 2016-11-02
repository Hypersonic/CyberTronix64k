EQU STDOUT 0x200
EQU STDIN 0x201

EQU S00 0x40
EQU S01 0x41
EQU S02 0x42
EQU S03 0x43
EQU S04 0x44
EQU S05 0x45
EQU S06 0x46
EQU S07 0x47
EQU S08 0x48
EQU S09 0x49

# void printhex(unsigned num) {
#   char out;
#   if (num == 0) {
#     write(STDOUT, '\0');
#			return;
#   }
#   while (num != 0) {
#     out = num;
#     out &= 0xF;
#     if (!(out < 10)) {
#       out += 'A' - 10;
#     } else {
#       out += '0';
#     }
#     write(STDOUT, out);
#     num >>= 4;
#   }
# }

EQU __a_printhex_num s00

EQU __c_printhex_0 s01
EQU __c_printhex_4 s02
EQU __c_printhex_10 s03
EQU __c_printhex_16 s04
EQU __c_printhex_ascii_0 s05
EQU __c_printhex_ascii_A s06

EQU __v_printhex_out s07

printhex:
	mi __c_printhex_0, 0
	mi __c_printhex_4, 4
	mi __c_printhex_10, 10
	mi __c_printhex_16, 16
	mi __c_printex_ascii_0, '0'
	mi __c_printex_ascii_A, 'A'

	jq __a_printhex_num, __c_printhex_zero, __l_printhex_loop_start
	  mi STDOUT, '0'
		ji __l_printhex_loop_end

	jq __a_printhex_num, __c_printhex_zero, __l_printhex_loop_end
		__l_printhex_loop_start:
			mv __v_printhex_out, num
			nd __v_printhex_out, __c_printhex_16

			jl __v_printhex_out, __c_printhex_10, __l_printhex_else
				__l_printhex_then:
					ad __v_printhex_out, __c_printhex_ascii_A
					ji __l_printhex_if_done
				__l_printhex_else:
					ad __v_printhex_out, __c_printhex_ascii_0
				__l_printhex_if_done:

			mv STDOUT, __v_printhex_out
			sr __v_printhex_num, __c_printhex_4
		  ji __l_printhex_loop_start
		__l_printhex_loop_end:

	ret
