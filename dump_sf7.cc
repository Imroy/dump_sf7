/*
        Copyright 2021 Ian Tester

        This file is part of dump_sf7.

        dump_sf7 is free software: you can redistribute it and/or modify
        it under the terms of the GNU General Public License as published by
        the Free Software Foundation, either version 3 of the License, or
        (at your option) any later version.

        dump_sf7 is distributed in the hope that it will be useful,
        but WITHOUT ANY WARRANTY; without even the implied warranty of
        MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
        GNU General Public License for more details.

        You should have received a copy of the GNU General Public License
        along with dump_sf7.  If not, see <http://www.gnu.org/licenses/>.
*/
#include "sf7.hh"
#include "charmaps.hh"
#include "BASIC.hh"
#include "wildcard.hh"
#include <iostream>
#include <fstream>
#include <unistd.h>
#include <sys/stat.h>

void usage(std::string progname) {
  std::cerr << progname << " [options] <image.sf7> [filenames or wildcards...]" << std::endl << std::endl;
  std::cerr << "Options:" << std::endl;
  std::cerr << "\t-l\tList filenames. No extraction is performed." << std::endl;
  std::cerr << "\t-j\tUse Japanese character map when converting text to UTF-8." << std::endl;
  std::cerr << "\t-r\tRaw output. Files are dumped as in the image with no" << std::endl
	    << "\t\tSega => UTF-8 conversion or BASIC detokenisation." << std::endl;
  std::cerr << "\t-b\tBASIC detokenisation of all non-ASCII files, not just" << std::endl
	    << "\t\tones named *.BAS." << std::endl;
  std::cerr << std::endl;
  std::cerr << "\tIf file names or wildcards are listed, only matching" << std::endl
	    << "\tfiles will be processed." << std::endl;
  std::cerr << std::endl;
}

int main(int argc, char* argv[]) {
  bool only_list = false, use_japanese = false, raw = false, all_basic = false;
  {
    int opt;
    while ((opt = getopt(argc, argv, "ljrb")) != -1) {
      switch (opt) {
      case 'l':
	only_list = true;
	break;

      case 'j':
	use_japanese = true;
	break;

      case 'r':
	raw = true;
	break;

      case 'b':
	all_basic = true;
	break;

      default:
	break;
      }
    }
  }
  if (optind >= argc) {
    usage(argv[0]);
    return -1;
  }

  SF7::Disk disk;
  {
    std::string filepath = argv[optind];
    std::ifstream ifs(filepath, std::ios::binary);
    if (!ifs.is_open())
      return -1;

    uint8_t buffer[SF7::track_size * 10];
    while (ifs.good()) {
      ifs.read(reinterpret_cast<char*>(buffer), sizeof(buffer));
      disk.load_data(buffer, ifs.gcount());
    }

    ifs.close();
  }

  if (disk.is_sys()) {
    std::cout << "System disk: " << disk.name() << std::endl;
    auto IPL = disk.IPL();
    std::ofstream ofs("IPL.bin", std::ios_base::out | std::ios_base::trunc);
    ofs.write(reinterpret_cast<char*>(IPL.data()), IPL.size());
    ofs.close();
    std::cout << "Wrote initial program loader to IPL.bin" << std::endl;
  }

  int num_wildcards = argc - optind - 1;
  std::string wildcards[num_wildcards];
  for (int i = optind + 1; i < argc; i++)
    wildcards[i - optind - 1] = argv[i];

  auto files = disk.list_directory(use_japanese ? Sega::japan_charmap : Sega::export_charmap);
  for (auto file : files) {
    auto filename = file.filename();

    if (num_wildcards > 0) {
      bool matches = false;
      for (int i = 0; i < num_wildcards; i++)
	if (strmatch(filename, wildcards[i])) {
	  matches = true;
	  break;
	}
      if (!matches)
	continue;
    }

    std::cout << filename << "\t" << SF7::file_type_names[static_cast<uint8_t>(file.filetype())];
    if (file.readonly())
      std::cout << "\tread-only";
    else
      std::cout << "\tread-write";

    if (only_list) {
      std::cout << std::endl;
      continue;
    }

    auto contents = file.read();

    if (!raw) {
      if (file.filetype() == SF7::File::type::ascii) {
	std::cout << "\t[Sega text]";
	contents = Sega::convert_utf8(contents, use_japanese ? Sega::japan_charmap : Sega::export_charmap);

      } else if ((file.filetype() == SF7::File::type::non_ascii)
		 && (all_basic
		     || ((filename.size() >= 4) && (filename.substr(filename.size() - 4, 4) == ".BAS"))
		     )
		 ) {
	std::cout << "\t[BASIC]";
	contents = BASIC::detokenise(contents, use_japanese ? Sega::japan_charmap : Sega::export_charmap);

      }
    }
    std::cout << std::endl;

    std::ofstream ofs(filename, std::ios_base::out | std::ios_base::trunc);
    ofs.write(reinterpret_cast<char*>(contents.data()), contents.size());
    ofs.close();

    if (file.readonly()) {
      struct stat stats;
      if (stat(filename.c_str(), &stats) == 0) {
	stats.st_mode &= ~(S_IWUSR | S_IWGRP | S_IWOTH);
	chmod(filename.c_str(), stats.st_mode);
      }
    }
  }

  return 0;
}
