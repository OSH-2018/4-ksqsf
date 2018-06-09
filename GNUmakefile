override CXXFLAGS += -O0 -Wall -Wextra -Werror
sources := $(wildcard *.cpp)
binaries := $(sources:%.cpp=%)

all: $(binaries)

clean:
	rm -f *.o $(binaries)
