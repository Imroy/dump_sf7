# Sega SF-7000 floppy file dumper

This program dumps files from a [Sega SF-7000](https://segaretro.org/Super_Control_Station_SF-7000) floppy disc image.

It is based on documentation in [the User's Manual](https://segaretro.org/images/d/dd/Super_Control_Station_AU_SF-7000_User%27s_Manual_fix.compressed.pdf) (pages 189 to 194).

## Dependencies

`dump_sf7` is a simple program with no dependencies. It only needs a C++11 compiler and CMake to build.

    apt-get install g++ cmake

## Building and installing

Use CMake and make in a `build` directory.

    mkdir build
    cd build
    cmake ..
    make
    sudo make install

## Command line usage

The program takes a single required argument; the name of a floppy disc image. It also has four options and any number of filenames or wildcards.

    dump_sf7 [options] <dump.sf7> [filenames or wildcards...]

Files are dumped to the current directory i.e where the program is run from.

- Files marked as type 'ASCII' are converted from the [Sega SC-3000 character set](https://en.wikipedia.org/wiki/Sega_SC-3000_character_set) ('export' version by default) to UTF-8. Note that some characters have no equivalent in Unicode however.
- Files marked as type 'non-ASCII' and having a filename ending in `.BAS` are detokenised as BASIC source. Any text is also converted from the Sega SC-3000 character set.
- Files marked as type 'HEX' are dumped as-is.

If one or more filenames or wildcards are provided after the image filename, only files matching any of them will be processed.

### Options

The `-l` option causes only the files to be listed. No dumping of contents is performed.

The `-j` option causes the Japanese character set to be used when converting text to UTF-8. The 'export' character set is used by default.

The `-r` option forces raw output. File data is copied exactly as it is from the image. None of the above conversions are performed.

The `-b` option forces BASIC detokenisation on all files marked 'non-ASCII', not just those with a filename ending with `.BAS`.

If both the last two options are given (either as `-r -b` or `-rb`), only raw output takes place.
