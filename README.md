# Sega SF-7000 floppy file dumper

This program dumps files from a [Sega SF-7000](https://segaretro.org/Super_Control_Station_SF-7000) floppy image.

It is based on documentation in [the User's Manual](https://segaretro.org/images/d/dd/Super_Control_Station_AU_SF-7000_User%27s_Manual_fix.compressed.pdf) (pages 189 to 194).

## Dependencies

`dump_sf7` is a simple program with no dependencies. It just needs a C++11 compiler and CMake to build.

    apt-get install g++ cmake

## Building and installing

Just use CMake and make in a `build` directory.

    mkdir build
    cd build
    cmake ..
    make
    sudo make install
