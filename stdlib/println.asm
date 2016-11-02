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

# struct str {
#   char* ptr;
#		size_t len;
# };
#
# 
# void print(char* ptr, size_t len) {
#   char* end = ptr;
#   end += len;
#   while (ptr != end) {
#     write(STDOUT, *ptr);
#     ptr++;
#   }
# }
# void println(char* ptr, size_t len) {
#   print(ptr, len);
#   write(STDOUT, '\n');
# }

EQU __a_print_ptr s00
EQU __a_print_len s01
EQU __v_print_end s02
EQU __v_print_one s03 ; instead of inc, just make a temporary reg once
print:
  mv __v_print_end, __a_print_ptr
	ad __v_print_end, __a_print_len
	mi __v_print_one, 1
	jq __v_print_ptr, __v_print_end, __l_print_loop_end
	  __l_print_loop_start:
		  md STDOUT, __v_print_ptr
			ad __v_print_ptr, __v_print_one
	  __l_print_loop_end:
	ret

EQU __a_println_ptr s00
EQU __a_println_len s01
EQU __v_println_nl s02
println:
  call print
	mi STDOUT, __v_println_nl
	ret
