.PHONY: linux macos windows build clean

# ---- Common ----
CXX         := g++
CXXFLAGS_COMMON := -std=c++11 -O3 -I. -Wno-address-of-packed-member
SRCS        := main.cpp calcwit.cpp fr.cpp pol.cpp
OBJS        := $(SRCS:.cpp=.o)
DEPS_HPP    := circom.hpp calcwit.hpp fr.hpp
BIN_WINDOWS := pol.exe
BIN_UNIX    := pol
BINS        := $(BIN_UNIX) $(BIN_WINDOWS)

# ---- Linux ----
linux: BIN=$(BIN_UNIX)
linux: CXXFLAGS=$(CXXFLAGS_COMMON)
linux: LDFLAGS=-static
linux: LDLIBS=-lgmp
linux: $(BIN_UNIX)

# ---- macOS ----
macos: BIN=$(BIN_UNIX)
macos: CXXFLAGS=$(CXXFLAGS_COMMON) -I/opt/homebrew/include -include gmp_patch.hpp
macos: LDFLAGS=-Wl,-search_paths_first -Wl,-dead_strip
macos: LDLIBS=/opt/homebrew/lib/libgmp.a
macos: $(BIN_UNIX)

# ---- Windows (MinGW) ----
windows: BIN=$(BIN_WINDOWS)
windows: CXXFLAGS=$(CXXFLAGS_COMMON) -I/include -Duint="unsigned int"
windows: LDFLAGS=-static
windows: LDLIBS=-L/lib -lgmp -lmman
windows: $(BIN_WINDOWS)

# ---- Rules ----
$(BINS): $(OBJS)
	$(CXX) $(LDFLAGS) $^ $(LDLIBS) -o $@

%.o: %.cpp $(DEPS_HPP)
	$(CXX) $(CXXFLAGS) -c $< -o $@

clean:
	rm -f $(OBJS) $(BIN_WINDOWS) $(BIN_UNIX)

