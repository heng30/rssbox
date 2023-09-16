#!/bin/bash

# How to build?
# 1. Download oepnssl source codes from https://github.com/openssl/openssl
# 2. Run `./config no-shared --prefix=/path/to/installation/directory` to configure it for static library build
# 3. make && make install
# 4. Set the below enviroment varibles: OPENSSL_DIR, OPENSSL_LIB_DIR, OPENSSL_INCLUDE_DIR
#
# 5. Download xcb source codes from https://xcb.freedesktop.org/
# 6. Run `./configure --enable-shared=no --enable-static --prefix=/path/to/installation/directory`
# 7 make && make install
# 8. set `RUSTFLAGS` varible for linker
#
# 10. Download QT5  source codes from https://www.qt.io/offline-installers
# 11. Run `./configure -static -prefix /path/to/installation/directory`
# 12. make && make install
# 13. set `RUSTFLAGS` varible for linker
#
# Reference:
# 1. building Qt5 troubleshot: https://www.jianshu.com/p/03badff773ff

export XCB_LIBRARY_PATH=/home/blue/static-lib/libxcb-1.15/lib
export QT5_LIBRARY_PATH=/home/blue/static-lib/qtbase-5.15.2/lib
export OPENSSL_DIR=/home/blue/static-lib/openssl-1.1.1
export OPENSSL_LIB_DIR=$OPENSSL_DIR/lib
export OPENSSL_INCLUDE_DIR=$OPENSSL_DIR/include
export HOST=x86_64-unknown-linux-gnu
export TARGET=x86_64-unknown-linux-musl

if ! [ -f "/bin/musl-g++" ]; then
    echo "link g++ -> musl-g++"
    sudo ln -s /bin/g++ /bin/musl-g++
fi

RUSTFLAGS="-L $XCB_LIBRARY_PATH -L $QT5_LIBRARY_PATH" SLINT_STYLE=fluent cargo build --release --target=x86_64-unknown-linux-musl

