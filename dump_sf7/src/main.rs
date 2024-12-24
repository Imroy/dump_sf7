/*
  dump_sf7
  Copyright 2023 Ian Tester

  This library is free software: you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  This library is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this library.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::fs::File;
use std::io::prelude::*;

extern crate getopts;
use getopts::Options;
use std::env;

use sega_basic::*;
use sc3000_charset::*;
use sf7000_fs::*;

/// Match an input string against a pattern.
///
/// Based on [this Wildcard Pattern Matching example](https://www.geeksforgeeks.org/wildcard-pattern-matching/) on GeeksforGeeks.
/// Converted to Rust.
///
/// Returns `true` if it matches.
pub fn strmatch(input: &str, pat: &str) -> bool {
    // Convert strings into vectors of characters
    // i.e UTF-8 bytes into Unicode code points
    let inchars: Vec<_> = input.chars().collect();
    let patchars: Vec<_> = pat.chars().collect();

    // empty pattern can only match with empty string
    if patchars.is_empty() {
        return inchars.is_empty();
    }

    // lookup table for storing results of subproblems
    let mut lookup = vec![ vec![ false; patchars.len() + 1 ]; inchars.len() + 1 ];

    // empty pattern can match with empty string
    lookup[0][0] = true;

    // Only '*' can match with empty string
    for j in 1..=patchars.len() {
        if patchars[j - 1] == '*' {
            lookup[0][j] = lookup[0][j - 1];
        }
    }

    // fill the table in bottom-up fashion
    for i in 1..=inchars.len() {
        for j in 1..=patchars.len() {
            // Two cases if we see a '*'
            // a) We ignore '*' character and move
            //    to next  character in the pattern,
            //     i.e., '' indicates an empty sequence.
            // b) '*' character matches with ith
            //     character in input
            if patchars[j - 1] == '*' {
                lookup[i][j] = lookup[i][j - 1] || lookup[i - 1][j];

                // Current characters are considered as
                // matching in two cases
                // (a) current character of pattern is '?'
                // (b) characters actually match
            } else if patchars[j - 1] == '?' || inchars[i - 1] == patchars[j - 1] {
                lookup[i][j] = lookup[i - 1][j - 1];

                // If characters don't match
            } else {
                lookup[i][j] = false;
            }
        }
    }

    lookup[inchars.len()][patchars.len()]
}

fn usage(progname: &str, opts: Options) {
    let brief = format!("Usage: {} [options] <image.sf7> [filenames or wildcards...]", progname);
    eprint!("{}", opts.usage(&brief));
    eprintln!();
    eprintln!("    If file names or wildcards are listed, only matching files will be processed.");
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let progname = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help menu.");
    opts.optflag("l", "list", "List filenames. No extraction is performed.");
    opts.optflag("j", "japanese", "Use Japanese character map when converting text to UTF-8. The 'export' character map is used by default.");
    opts.optflag("r", "raw", "Raw output. Files are dumped as in the image with no Sega => UTF-8 conversion or BASIC detokenisation.");
    opts.optflag("b", "basic", "BASIC detokenisation of all non-ASCII files, not just ones named *.BAS.");
    opts.optflag("t", "tape", "Detokenise BASIC using the list of statements and functions available to Sega SC-3000 BASIC Level 2 or 3 (on cartridge)");

    let matches = opts.parse(&args[1..])
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    if matches.opt_present("h") || matches.free.is_empty() {
        usage(&progname, opts);
        return Ok(());
    }

    let only_list = matches.opt_present("l");
    let cset = if matches.opt_present("j") { CharacterSet::Japanese } else { CharacterSet::Export };
    let raw = matches.opt_present("r");
    let all_basic = matches.opt_present("b");
    let basic_ver = if matches.opt_present("t") { SegaBasicVersion::CartridgeBasic } else { SegaBasicVersion::DiskBasic };

    let mut disk = Disk::new();
    {
        let mut file = File::open(matches.free[0].clone())?;
        let mut disk_buf = Vec::new();
        file.read_to_end(&mut disk_buf)?;
        disk.load_data(&disk_buf);
    }

    let wildcards = &matches.free[1..];

    if !only_list && disk.is_sys() {
        let mut matches = true;
        if !wildcards.is_empty() {
            matches = false;
            for wc in wildcards {
                if strmatch("IPL.bin", wc.as_str()) {
                    matches = true;
                    break;
                }
            }
        }

        if matches {
            println!("System disk: {}", disk.name(cset));
            let mut file = File::create("IPL.bin")?;
            file.write_all(disk.ipl())?;
            println!("Wrote initial program loader to IPL.bin");
        }
    }

    let files = disk.list_directory(cset);
    for file in files {
        let mut matches = false;
        if !wildcards.is_empty() {
            for wc in wildcards {
                if strmatch(file.name.as_str(), wc.as_str()) {
                    matches = true;
                    break;
                }
            }
            if !matches {
                continue;
            }
        }

        print!("{}\t{}\tread-{}", file.name, file.file_type, if file.readonly { "only" } else { "write" });

        if only_list {
            println!();
            continue;
        }

        let contents = file.read();
        let mut outfile = File::create(&file.name)?;
        if !raw {
            let sc3kstr = SC3000String::from_cset(&contents, cset);
            if file.file_type == FileType::Ascii {
                print!("\t[Sega text]");
                outfile.write_all(sc3kstr.to_string().as_bytes())?;
            } else if file.file_type == FileType::NonAscii
                && (all_basic || (file.name.len() >= 4 && &file.name[file.name.len()-4..] == ".BAS")) {
                    print!("\t[BASIC]");
                    outfile.write_all(detokenise(&sc3kstr, basic_ver).as_bytes())?;
                } else {
                    print!("\t[Raw]");
                    outfile.write_all(&contents)?;
                }
        } else {
            print!("\t[Raw]");
            outfile.write_all(&contents)?;
        }

        if file.readonly {
            if let Ok(m) = outfile.metadata() {
                let mut perms = m.permissions();
                perms.set_readonly(true);
                outfile.set_permissions(perms)?;
            }
        }

        println!();
    }

    Ok(())
}
