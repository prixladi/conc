CC=gcc
FLAGS = -D_GNU_SOURCE -D_POSIX_C_SOURCE=200112L -W -Wall -pedantic -Werror -std=c99 -Wno-gnu-auto-type
SOURCES = src/external/*.c src/utils/*.c src/*.c
HEADERS = src/external/*.h src/utils/*.h src/*.h

.PHONY: build install run

build_debug_address: FLAGS += -fsanitize=undefined,address -g -D __DEBUG__
build_debug_address: 
	$(CC) $(FLAGS) $(SOURCES) -o ./build/concd-debug

run_debug_address: build_debug_address
	./build/concd-debug -w ./run

build_debug_thread: FLAGS += -fsanitize=undefined,thread -g -D __DEBUG__
build_debug_thread: 
	$(CC) $(FLAGS) $(SOURCES) -o ./build/concd-debug

run_debug_thread: build_debug_thread
	./build/concd-debug -w ./run

build: FLAGS += -O2
build: 
	$(CC) $(FLAGS) $(SOURCES) -o ./build/concd

run: build
	./build/concd -w ./run

install: build
	./install.sh

format: 
	clang-format -i $(SOURCES) $(HEADERS)

tidy: 
	clang-tidy $(SOURCES) $(HEADERS)