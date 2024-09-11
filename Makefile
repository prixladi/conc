CC=gcc
FLAGS = -D_POSIX_C_SOURCE=199309L -W -Wall -pedantic -Werror -std=c99
SOURCES = src/external/*.c src/utils/*.c src/*.c

build: FLAGS += -fsanitize=address -g -D __DEBUG__
build: 
	$(CC) $(FLAGS) $(SOURCES) -o ./run/conc

build_release: 
	$(CC) $(FLAGS) $(SOURCES) -o ./run/conc-r

run: build
	./run/conc

dbg: build
	gdb ./run/conc