equ ip_ ip
# comment1
; comment2

label1:
mv sc, sc
mv sc, sc
label2:
mv sc, sc
mv sc, sc
mv sc, sc
label3:
mi ip, label1
mi ip_, label2
jg ip, ip_, label3
