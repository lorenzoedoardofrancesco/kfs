RELEASE_TARGET=target/i386-unknown-none/release

all: release

release:
	cargo build --release
	objcopy -O binary $(RELEASE_TARGET)/kfs-1 isofiles/boot/kfs.bin
	ld -n -o isofiles/boot/kfs.bin -T linker.ld src/boot/boot.o src/boot/multiboot_header.o
	grub-mkrescue -o isofiles/kfs.iso isofiles

clean:
	cargo clean
	rm -rf isofiles/boot/kfs.bin
	rm -rf isofiles/kfs.iso

.PHONY: all debug release debug_bin release_bin clean