run:
	cargo bootimage
	qemu-system-x86_64 -drive format=raw,file=target/fiddle/debug/bootimage-fiddle_os.bin

release:
	cargo bootimage --release
	qemu-system-x86_64 -drive format=raw,file=target/fiddle/debug/bootimage-fiddle_os.bin