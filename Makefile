RELEASE_TARGET=target/i386-unknown-none/release

all: release

release:
	cargo build --release
	objcopy -O binary $(RELEASE_TARGET)/kfs-1 boot/kfs.bin
	grub-mkrescue -o boot/kfs.iso boot

clean:
	cargo clean
	rm -rf boot/kfs.bin
	rm -rf boot/kfs.iso

.PHONY: all debug release debug_bin release_bin clean