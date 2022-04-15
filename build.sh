#!/bin/bash

build_debug () {
    cargo build && cp target/debug/arena .
}

build_release () {
    cargo build -r && cp target/debug/arena .
}

if [ 0 -eq $# ]; then build_debug;
elif [ 1 -eq $# ]; then if [ "-r" = $1 ] || [ "--release" = $1 ]; then build_release;
    elif [ "-d" = $1 ] || [ "--debug" = $1 ]; then build_debug; else echo  "Usage: ./build.sh [-r|-d]"; fi;
else echo "Usage: ./build.sh [-r|-d]"; fi
