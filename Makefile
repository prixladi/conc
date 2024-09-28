CC=clang
FLAGS = -D_POSIX_C_SOURCE=200112L -W -Wall -pedantic -Werror -std=c99 -Wno-gnu-auto-type
SOURCES = src/external/*.c src/utils/*.c src/*.c
HEADERS = src/external/*.h src/utils/*.h src/*.h

.PHONY: install

run: run_release

build_debug: FLAGS += -fsanitize=undefined,address -g -D __DEBUG__
build_debug: 
	$(CC) $(FLAGS) $(SOURCES) -o ./build/conc-debug

run_debug: build_debug
	./build/conc-debug

build_debug_t: FLAGS += -fsanitize=undefined,thread -g -D __DEBUG__
build_debug_t: 
	$(CC) $(FLAGS) $(SOURCES) -o ./build/conc-debug

run_debug_t: build_debug_t
	./build/conc-debug

build: FLAGS += -O2
build_release: 
	$(CC) $(FLAGS) $(SOURCES) -o ./build/conc

run_release: build_release
	./build/conc

install: build_release
	sudo ./install.sh

format: 
	clang-format -i $(SOURCES) $(HEADERS)

tidy: 
	clang-tidy $(SOURCES) $(HEADERS)