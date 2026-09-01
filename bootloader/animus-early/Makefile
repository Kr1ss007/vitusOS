CC ?= gcc
CFLAGS ?= -O3 -Wall -Wextra -std=c11 $(shell pkg-config --cflags libdrm libpipewire-0.3)
LDFLAGS ?= $(shell pkg-config --libs libdrm libpipewire-0.3)

TARGET = animus-early

all: $(TARGET)

$(TARGET): animus-early.c
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

clean:
	rm -f $(TARGET)

.PHONY: all clean
