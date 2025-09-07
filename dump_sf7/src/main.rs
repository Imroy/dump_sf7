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

use sc3000_charset::*;
use sf7000_fs::*;

fn usage(progname: &str, opts: Options) {
    let brief = format!("Usage: {} [options] <image.sf7> [filenames or wildcards...]", progname);
    eprint!("{}", opts.usage(&brief));
    eprintln!();
    eprintln!("    If file names or wildcards are listed, only matching files will be processed.");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let progname = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help menu.");
    opts.optflag("l", "list", "List filenames. No extraction is performed.");
    opts.optflag("j", "japanese", "Use Japanese character map when converting text to UTF-8. The 'export' character map is used by default.");
    opts.optflag("r", "raw", "Raw output. Files are dumped as in the image with no Sega => UTF-8 conversion or BASIC detokenisation.");
    opts.optflag("b", "basic", "BASIC detokenisation of all non-ASCII files, not just ones named *.BAS.");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!("{}", f.to_string()) }
    };
    if matches.opt_present("h") || matches.free.is_empty() {
        usage(&progname, opts);
        return;
    }

    let only_list = matches.opt_present("l");
    let charmap = if matches.opt_present("j") { gen_japanese_charmap() } else { gen_export_charmap() };
    let raw = matches.opt_present("r");
    let all_basic = matches.opt_present("b");

    let mut disk = Disk::new();
    {
        let mut file = File::open(matches.free[0].clone()).unwrap();
        let mut disk_buf = Vec::new();
        file.read_to_end(&mut disk_buf);
        disk.load_data(&disk_buf);
    }

    if disk.is_sys() {
        println!("System disk: {}", disk.name(&charmap));
        let mut file = File::create("IPL.bin").unwrap();
        file.write_all(&disk.ipl());
        println!("Wrote initial program loader to IPL.bin");
    }

    let files = disk.list_directory(&charmap);
    for file in files {
        print!("{}\t{}\tread-{}", file.filename, file.file_type, if file.readonly { "only" } else { "write" });

        if only_list {
            println!("");
            continue;
        }
    }
}
