CARGO := xargo
AS := as
LD := ld
OBJCOPY := objcopy
QEMU := qemu-system-x86_64 

ARCH := x86_64
LDFLAGS := -T src/arch/$(ARCH)/link.ld -n --gc-sections
ASFLAGS := -64
QEMUFLAGS := -m 64M -hda build/main.img -hdb build/fs.img -serial mon:stdio -s -no-reboot
QEMUFLAGS += -net user -net nic,model=e1000 
#QEMUFLAGS += -nographic

include src/arch/$(ARCH)/build.mk

.PHONY: kernel clean image qemu

all: build build/startup.o $(AS_OBJECTS) build/main.elf

qemu: image
	$(QEMU) $(QEMUFLAGS)

image: build/main.img

build/main.img: build/main.elf grub.cfg
	mkdir -p iso/boot/grub/
	cp grub.cfg iso/boot/grub/
	cp build/main.elf iso/boot/main.elf
	grub-mkrescue -o $@ iso/

build/%.o: src/arch/$(ARCH)/%.S
	$(AS) $(ASFLAGS) -c $< -o $@

build/main.elf: kernel $(OBJECTS)
	$(LD) $(LDFLAGS) $(OBJECTS) target/x86_64-unknown-none/release/libparados.a -o $@

build:
	mkdir -p build

kernel:
	RUST_TARGET_PATH=$(shell pwd) $(CARGO) build --target x86_64-unknown-none --release

clean:
	$(CARGO) clean
	rm -rf build/ iso/

