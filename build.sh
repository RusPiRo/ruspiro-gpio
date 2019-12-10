#!/bin/sh

set +ev

if [ $# -eq 0 ] 
    then 
        echo "not enough parameter given"
        exit 1
fi

# check which aarch version to build
if [ $1 = "64" ]
    then
        # aarch64
        export CFLAGS='-march=armv8-a -Wall -O3 -nostdlib -nostartfiles -ffreestanding -mtune=cortex-a53'
        export RUSTFLAGS='-C linker=aarch64-elf-gcc -C target-cpu=cortex-a53 -C target-feature=+strict-align,+a53,+fp-armv8,+neon -C link-arg=-nostartfiles -C opt-level=3 -C debuginfo=0'
        if [ "$2" = "" ]
            then
                export CC='aarch64-elf-gcc'
                export AR='aarch64-elf-ar'
        fi
        cargo xbuild --target aarch64-unknown-linux-gnu --release
elif [ $1 = "32" ]
    then
        # aarch32
        export CFLAGS='-mfpu=neon-fp-armv8 -mfloat-abi=hard -march=armv8-a -Wall -O3 -nostdlib -nostartfiles -ffreestanding -mtune=cortex-a53'
        export RUSTFLAGS='-C linker=arm-eabi-gcc.exe -C target-cpu=cortex-a53 -C target-feature=+strict-align,+a53,+fp-armv8,+v8,+vfp3,+d16,+thumb2,+neon -C link-arg=-nostartfiles -C opt-level=3 -C debuginfo=0'
        if [ -z "$2" ]
            then
                export CC='arm-eabi-gcc.exe'
                export AR='arm-eabi-ar.exe'
        fi
        cargo xbuild --target armv7-unknown-linux-gnueabihf --release
else
    echo 'provide the archtitecture to be build. Use either "build.sh 32" or "build.sh 64"'
fi
