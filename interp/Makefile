run: interp
	./interp ../test_program

clean:
	rm ./interp

interp: interp.cpp memory.cpp memory.h
	gcc --std=c++11 $(shell pkg-config --cflags sdl2) interp.cpp memory.cpp -o interp $(shell pkg-config --libs sdl2)
