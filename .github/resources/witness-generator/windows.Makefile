CC = g++
CFLAGS = -std=c++11 -O3 -I. -I/include -Duint="unsigned int"
LDFLAGS = -L/lib -lgmp -lmman -static

DEPS_HPP = circom.hpp calcwit.hpp fr.hpp pol.cpp
DEPS_O = main.o calcwit.o fr.o pol.o

all: pol.exe

%.o: %.cpp $(DEPS_HPP)
	$(CC) -Wno-address-of-packed-member -c $< $(CFLAGS) -o $@

pol.exe: $(DEPS_O)
	$(CC) -o pol.exe $(DEPS_O) $(LDFLAGS)

clean:
	rm -f *.o pol.exe
