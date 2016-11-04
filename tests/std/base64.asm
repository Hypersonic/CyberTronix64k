import constants

public base64enc
public base64dec

#     | 0 1 2 3 4 5 6 7 8 9 A B C D E F
# ----+--------------------------------
# 0x0 | 0 1 2 3 4 5 6 7 8 9 A B C D E F
# 0x1 | G H I J K L M N O P Q R S T U V
# 0x2 | W X Y Z a b c d e f g h i j k l
# 0x3 | m n o p q r s t u v w x y z _ .

__base64_LOOKUP_TABLE:
data "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_."

# NOTE: length of to_write must be three times length of to_translate
# str base64enc(uint16_t* to_translate, size_t length, char* to_write)
equ __a_base64enc_to_translate s00
equ __a_base64enc_length s01
equ __a_base64enc_to_write s02

equ __r_base64enc_ptr s00
equ __r_base64enc_length s01

equ __c_base64enc_1 s03

equ __v_base64enc_to_write s04
equ __v_base64enc_tmp s05
equ __v_base64enc_start s06
equ __v_base64enc_end s07
equ __v_base64enc_curr s08
equ __v_base64enc_once_curr s09
equ __v_base64enc_once_out s10

base64enc:
	mi __c_base64enc_1, 1

	# to_write_start = to_write;
	mv __v_base64enc_to_write, __a_base64enc_to_write
	# start = to_translate
	mv __v_base64enc_start, __a_base64enc_to_translate
  # end = to_translate;
	mv __v_base64enc_end, __a_base64enc_to_translate
	# end += length;
	ad __v_base64enc_end, __a_base64enc_length
	# while (start != end) {
	__l_base64enc_loop_test:
	jq __v_base64enc_start, __v_base64enc_end, __l_base64enc_loop_end
		__l_base64enc_loop_start:
	#   curr = *start;
			md __v_base64enc_curr, __v_base64enc_start
	#   goto enc_num
			ji __l_base64enc_enc_num
			__l_base64enc_enc_num_end:
	#   start++;
			ad __v_base64enc_start, __c_base64enc_1
	# }
			ji __l_base64enc_loop_test
		__l_base64enc_loop_end:
	# return;
	mv __r_base64enc_ptr, __a_base64enc_to_write
	mv __r_base64enc_length, __v_base64enc_to_write
	sb __r_base64enc_length, __r_base64enc_ptr
	ret

	__l_base64enc_enc_num:
	  # once_curr = curr;
		mv __v_base64enc_once_curr, __v_base64enc_curr
		# once_curr &= 63;
		mi __v_base64enc_tmp, 63
		nd __v_base64enc_once_curr, __v_base64enc_tmp
		# out = __base64_LOOKUP_TABLE;
		mi __v_base64enc_once_out, __base64_LOOKUP_TABLE
		# out += once_curr;
		ad __v_base64enc_once_out, __v_base64enc_once_curr
		# *to_write = *out;
		md __v_base64enc_tmp, __v_base64enc_once_out
		ld __v_base64enc_to_write, __v_base64enc_tmp
		# to_write++;
		ad __v_base64enc_to_write, __c_base64enc_1

		# once_curr = curr;
		mv __v_base64enc_once_curr, __v_base64enc_curr
		# once_curr >>= 6;
		mi __v_base64enc_tmp, 6
		sr __v_base64enc_once_curr, __v_base64enc_tmp
		# once_curr &= 63;
		mi __v_base64enc_tmp, 63
		nd __v_base64enc_once_curr, __v_base64enc_tmp
		# out = __base64_LOOKUP_TABLE;
		mi __v_base64enc_once_out, __base64_LOOKUP_TABLE
		# out += once_curr;
		ad __v_base64enc_once_out, __v_base64enc_once_curr
		# *to_write = *out;
		md __v_base64enc_tmp, __v_base64enc_once_out
		ld __v_base64enc_to_write, __v_base64enc_tmp
		# to_write++;
		ad __v_base64enc_to_write, __c_base64enc_1

		# once_curr = curr;
		mv __v_base64enc_once_curr, __v_base64enc_curr
		# once_curr >>= 12;
		mi __v_base64enc_tmp, 12
		sr __v_base64enc_once_curr, __v_base64enc_tmp
		# once_curr &= 15;
		mi __v_base64enc_tmp, 15
		nd __v_base64enc_once_curr, __v_base64enc_tmp
		# out = __base64_LOOKUP_TABLE;
		mi __v_base64enc_once_out, __base64_LOOKUP_TABLE
		# out += once_curr;
		ad __v_base64enc_once_out, __v_base64enc_once_curr
		# *to_write = *out;
		md __v_base64enc_tmp, __v_base64enc_once_out
		ld __v_base64enc_to_write, __v_base64enc_tmp
		# to_write++;
		ad __v_base64enc_to_write, __c_base64enc_1

		# goto enc_num_end
		ji __l_base64enc_enc_num_end

#base64dec:
