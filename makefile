#*****************************************************************
# Makefile to build the RusPiRo crate
# setting the necessary environment for cross compiling on windows
#
# Copyright (c) 2019 by the authors
# 
# Author: André Borrmann 
# License: Apache License 2.0
#******************************************************************
CARGO_BIN = "$(USER_ROOT)\\.cargo\\bin"
ARM_GCC_BIN = "$(USER_ROOT)\\arm-gcc\\gcc-arm-eabi\\bin"
ARM_GCC_LIB = "$(USER_ROOT)\\arm-gcc\\gcc-arm-eabi\\lib\\gcc\\arm-eabi\\8.3.0"
TARGET = armv7-unknown-linux-gnueabihf
TARGETDIR = target\\armv7-unknown-linux-gnueabihf\\release
# environment variables needed by cargo xbuild to use the custom build target
export CC = arm-eabi-gcc.exe
export AR = arm-eabi-ar.exe
export CFLAGS = -std=c11 -mfpu=neon-fp-armv8 -mfloat-abi=hard -march=armv8-a -Wall -O3 -nostartfiles -ffreestanding -mtune=cortex-a53
export PATH +=  "$(PROJECT_ROOT);$(ARM_GCC_BIN);$(ARM_GCC_LIB);$(CARGO_BIN)"

# build the current crate
all:  
# update dependend crates to their latest version if any
	cargo update
# cross compile the crate
	cargo xbuild --target $(TARGET) --release

doc:
	# update dependend crates to their latest version if any
	cargo update
	# build docu for this crate using custom target
	xargo doc --all --no-deps --target $(TARGET) --release --open
	
test:
	xargo test --doc --target $(TARGET)

publish-dry-run:
	xargo publish --dry-run --target $(TARGET)

publish:
	xargo publish --target $(TARGET)

clean:
	cargo clean