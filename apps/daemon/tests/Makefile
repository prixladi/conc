CC=gcc
FLAGS = -D_GNU_SOURCE -D_POSIX_C_SOURCE=200112L -W -Wall -pedantic -std=c99 -Wno-gnu-auto-type -fsanitize=undefined,address -g -D __DEBUG__
SOURCES = ../src/utils/*.c ./utils/*.c main.c

.PHONY: build install run

build: 
	$(CC) $(FLAGS) $(SOURCES) -o ./build/concd-tests

run: build
	./build/concd-tests