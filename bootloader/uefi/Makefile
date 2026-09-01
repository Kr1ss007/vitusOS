# ==============================================================================
# AnimusBoot Stage 0 UEFI Bootloader Makefile (GNU-EFI / Clang)
# Target: x86_64-unknown-windows / EFI PE32+ executable
# ==============================================================================

ARCH            = x86_64
TARGET          = AnimusBoot.efi
SRC             = src/AnimusBoot.c
OBJS            = src/AnimusBoot.o

EFIINC          = /usr/include/efi
EFIINCS         = -I$(EFIINC) -I$(EFIINC)/$(ARCH) -I$(EFIINC)/protocol -Iinclude
EFILIB          = /usr/lib
EFI_CRT_OBJS    = $(EFILIB)/crt0-efi-$(ARCH).o
EFI_LDS         = $(EFILIB)/elf_$(ARCH)_efi.lds

CFLAGS          = $(EFIINCS) -fno-stack-protector -fpic -fshort-wchar -mno-red-zone \
                  -Wall -Wextra -Werror -DHAVE_USE_MS_ABI -O2 -fno-builtin

LDFLAGS         = -nostdlib -znocombreloc -T $(EFI_LDS) -shared \
                  -Bsymbolic -L $(EFILIB) $(EFI_CRT_OBJS)

LIBS            = -lefi -lgnuefi

all: $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

AnimusBoot.so: $(OBJS)
	$(LD) $(LDFLAGS) $(OBJS) -o $@ $(LIBS)

$(TARGET): AnimusBoot.so
	objcopy -j .text -j .sdata -j .data -j .dynamic -j .dynsym -j .rel \
	        -j .rela -j .rel.* -j .rela.* -j .reloc \
	        --target=efi-app-$(ARCH) $< $@
	@echo "Generated UEFI Application: $@"

clean:
	rm -f $(OBJS) AnimusBoot.so $(TARGET)

.PHONY: all clean
