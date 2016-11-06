run: interp
	./interp ../t

interp: interp.cpp memory.h
	g++ -Wall -Wextra --std=c++11 interp.cpp -o interp

clean:
	rm ./interp
