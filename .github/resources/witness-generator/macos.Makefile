CC = g++
CFLAGS = -std=c++11 -O3 -I. -I/opt/homebrew/include -include gmp_patch.hpp -Wno-address-of-packed-member
LDFLAGS = -Wl,-search_paths_first -Wl,-dead_strip -L/opt/homebrew/lib
LDLIBS = /opt/homebrew/lib/libgmp.a

DEPS_HPP = circom.hpp calcwit.hpp fr.hpp pol.cpp
DEPS_O = main.o calcwit.o fr.o pol.o

all: pol

%.o: %.cpp $(DEPS_HPP)
	$(CC) $(CFLAGS) -c $< -o $@

pol: $(DEPS_O)
	$(CC) -o $@ $(DEPS_O) $(LDFLAGS) $(LDLIBS)

clean:
	rm -f *.o pol

