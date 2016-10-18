JI start

equ PIXELS_WRITTEN, 0x201
equ ROWS_WRITTEN, 0x202
equ COLOR, 0x203
equ ITERATIONS, 0x204
equ PRINT, 0x5

start:
    MI 0xFE, 0x1 ; disable drawing
loop:
    MV 0x100, COLOR
    ADI COLOR, 0x01
    ; increment pointer
    ADI 0xFF, 0x01
    AD PIXELS_WRITTEN, SC
    MI SC, 0x1FF
    JQ PIXELS_WRITTEN, SC, redraw
    JI loop

redraw:
    MI PIXELS_WRITTEN, 0x00
    ADI ROWS_WRITTEN, 0x01
    ADI 0xFF, 0x01
    MI SC, 0x7f ; end of screen
    JQ ROWS_WRITTEN, SC, end
    JI start

; inf loop at end
end:
    ; Flip drawing on, then back off
    MI 0xFE, 0x0
    MI 0xFE, 0x1
    MI PIXELS_WRITTEN, 0x0
    MI ROWS_WRITTEN, 0x0
    ADI ITERATIONS, 0x6
    MV COLOR, ITERATIONS
    MI 0xFF, 0x0
    JI start
