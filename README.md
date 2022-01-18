# Sega SF-7000 floppy file dumper

This program dumps files from a [Sega SF-7000](https://segaretro.org/Super_Control_Station_SF-7000) floppy disc image.

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

## Command line usage

The program takes a single argument; the name of a floppy disc image. It also has two options.

    dump_sf7 [-r|-b] <dump.sf7>

The `-r` option forces raw output. File data is copied exactly as it is from the image. No conversion from the Sega character set to UTF-8 is performed on text, neither is BASIC detokenisation.

The `-b` option forces BASIC detokenisation on all files marked 'non-ASCII'. Normally the filename also has to end with '.BAS' for detokenisation to be performed.

If both options are given (either as `-r -b` or `-rb`), only raw output takes place.

Files are dumped to the current directory i.e where the program is run from.
