import constants

global getline

# struct str {
#	  char* ptr;
#   size_t len;
# };
#
# str getline(char* ptr, size_t capacity) {
#   char* start = ptr;
#   char* end = ptr;
#   end += capacity;
#   while (ptr != end) {
#     char c = read(STDIN);
#     if (c == '\n') {
#       break;
#     }
#     *ptr = c;
#     ptr++;
#   }
#   size_t len = end - ptr;
#   return { ptr, len };
# }

; takes (ptr, len):
;     (s00, s01)
EQU __a_getline_capacity s01
; returns (ptr, len):
;     (s00, s01)
EQU __ar_getline_ptr s00
EQU __r_getline_len s01
; stack variables:
EQU __v_getline_start s02
EQU __v_getline_end s03
EQU __v_getline_c s04
EQU __v_getline_nl s05
EQU __v_getline_one s06
getline:
	mi __v_getline_nl, '\n'
  mv __v_getline_start, __ar_getline_ptr
  mv __v_getline_end, __ar_getline_ptr
  ad __v_getline_end, __a_getline_capacity
  jq __v_getline_start, __v_getline_end, __l_getline_loop_break
		__v_getline_loop_start:
			mv __v_getline_c, STDIN
			jq __v_getline_c, __v_getline_nl, __l_getline_loop_break
			ld __v_getline_start, __v_getline_c
			ad __v_getline_start, __v_getline_one
			ji __v_getline_loop_start
		__l_getline_loop_break:
  mv __r_getline_len, __v_getline_end
  sb __r_getline_len, __ar_getline_ptr
  ret
