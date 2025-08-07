CC = g++
CFLAGS = -std=c++11 -O3 -I. -I/opt/homebrew/include -include gmp_patch.hpp
LDFLAGS = -L/opt/homebrew/lib -lgmp

DEPS_HPP = circom.hpp calcwit.hpp fr.hpp pol.cpp
DEPS_O = main.o calcwit.o fr.o pol.o

all: pol

%.o: %.cpp $(DEPS_HPP)
	$(CC) -Wno-address-of-packed-member -c $< $(CFLAGS) -o $@

pol: $(DEPS_O)
	$(CC) -o pol $(DEPS_O) $(LDFLAGS)

clean:
	rm -f *.o pol
