equ PIXELS_WRITTEN, 0x201
equ ROWS_WRITTEN, 0x202
equ COLOR, 0x203

MI 0xFE, 0x1 ; disable drawing
start:
    MV 0x100, COLOR
    ADI COLOR, 0x01
    ; increment pointer
    ADI 0xFF, 0x01
    AD PIXELS_WRITTEN, SC
    MI SC, 0x1FF
    JQ PIXELS_WRITTEN, SC, redraw
    MI 0x0, start

redraw:
    ; Flip drawing off, then on
    MI 0xFE, 0x0
    MI 0xFE, 0x1
    MI PIXELS_WRITTEN, 0x00
    ADI ROWS_WRITTEN, 0x01
    ADI 0xFF, 0x01
    MI SC, 0x7f ; end of screen
    JQ ROWS_WRITTEN, SC, end
    MI 0x0, start

; inf loop at end
end:
    MI 0x0, end
