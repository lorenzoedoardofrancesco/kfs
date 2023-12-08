RELEASE_TARGET=target/i386-unknown-none/release

all: release

release:
	cargo build --release --target=i386-unknown-none.json
	mkdir -p build
	nasm -f elf32 src/boot/boot.asm -o build/boot.o
	nasm -f elf32 src/boot/multiboot_header.asm -o build/multiboot_header.o
	ld -m elf_i386 -n -o isofiles/boot/kfs.bin -T linker.ld build/multiboot_header.o build/boot.o $(RELEASE_TARGET)/libkfs_1.a
	grub-mkrescue -o isofiles/kfs.iso isofiles

clean:
	cargo clean
	@rm -rf isofiles/boot/kfs.bin
	@rm -rf isofiles/kfs.iso
	@rm -rf build

re: clean all

.PHONY: all clean re release