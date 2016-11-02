EQU STDOUT 0x200
EQU STDIN 0x201

EQU S00 0x40
EQU S01 0x41
EQU S02 0x42
EQU S03 0x43

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
; returns (ptr, len):
;     (s00, s01)
EQU __a_getline_ptr s00
EQU __a_getline_capacity s01
; stack variables:
EQU __v_getline_start s02
EQU __v_getline_end s03
EQU __v_getline_c s04
EQU __v_getline_len s05
getline:
  mv __v_getline_start, __a_getline_ptr
  mv __v_getline_end, __a_getline_ptr
  ad __v_getline_end, __a_getline_capacity
  jq __v_getline_start, __v_getline_end, __l_getline_loop_break
    mv __v_getline_c, STDIN
    jq __v_getline_c, '\n', __l_getline_loop_break
    ld __v_getline_start, __v_getline_c
    inc __v_getline_start
  __l_getline_loop_break:
  mv __v_getline_len, __v_getline_end
  sb __v_getline_len, __a_getline_ptr
  ret
