# Makefile template for shared library

CC = gcc # C compiler
CFLAGS = -fPIC -Wall -Wextra -O2 -g # C flags
LDFLAGS = -shared  # linking flags
RM = rm -f  # rm command
TARGET_LIB = cplugin# target lib

SRCS = plugin.c # source files
OBJS = $(SRCS:.c=.o)

native: CC = gcc
#native: TARGET_LIB += .so
native: ${TARGET_LIB}.so

#cross: TARGET_LIB =
cross: CC = x86_64-w64-mingw32-gcc
cross: ${TARGET_LIB}.dll

.PHONY: all
all: ${TARGET_LIB}.so

%.dll: $(OBJS)
	$(CC) ${LDFLAGS} -o $@ $^

%.so: $(OBJS)
	$(CC) ${LDFLAGS} -o $@ $^

$(SRCS:.c=.d):%.d:%.c
	$(CC) $(CFLAGS) -MM $< >$@

include $(SRCS:.c=.d)

.PHONY: clean
clean:
	-${RM} ${TARGET_LIB}.* ${OBJS} $(SRCS:.c=.d)

