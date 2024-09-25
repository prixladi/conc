CC=gcc
FLAGS = -D_POSIX_C_SOURCE=200112L -W -Wall -pedantic -Werror -std=c99
SOURCES = src/external/*.c src/utils/*.c src/*.c

.PHONY: build install

build: FLAGS += -fsanitize=address -g -D __DEBUG__
build: 
	$(CC) $(FLAGS) $(SOURCES) -o ./build/conc-debug

run: build
	./build/conc-debug -l 5

dbg: build
	gdb ./build/conc-debug

build: FLAGS += -O2
build_release: 
	$(CC) $(FLAGS) $(SOURCES) -o ./build/conc

run_release: build_release
	./build/conc

install: build_release
	sudo ./install.sh