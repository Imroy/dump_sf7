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
#include <iostream>
#include <fstream>

int main(int argc, char* argv[]) {
  if (argc < 2) {
    std::cerr << argv[0] << " <image.sf7>" << std::endl << std::endl;
    return -1;
  }

  SF7::Disk disk;
  {
    std::string filepath = argv[1];
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
    std::ofstream ofs("IPL.bin", std::ios_base::out);
    ofs.write(reinterpret_cast<char*>(IPL.data()), IPL.size());
    ofs.close();
    std::cout << "Wrote initial program loader to IPL.bin" << std::endl;
  }

  auto files = disk.list_directory();
  for (auto file : files) {
    auto filename = file.filename();

    // Remove spaces at end of the two parts of the filename
    filename = filename.substr(0, filename.find_last_not_of(" ", 7) + 1)
      + filename.substr(8, filename.find_last_not_of(" ", 11) - 7);

    // Replace slashes (/) with a double dash (--)
    for (std::string::size_type pos{}, count{};
	 filename.npos != (pos = filename.find("/", pos, 1));
         pos++, ++count) {
      filename.replace(pos, 1, "--", 2);
    }

    std::cout << filename << "\t" << SF7::file_type_names[static_cast<uint8_t>(file.filetype())];
    if (file.readonly())
      std::cout << "\tread-only";
    else
      std::cout << "\tread-write";
    std::cout << std::endl;

    auto contents = file.read();
    std::ofstream ofs;
    ofs.open(filename, std::ios_base::out);

    if (file.filetype() == SF7::File::type::ascii) {
      auto utf8 = Sega::convert_utf8_export(contents);
      ofs.write(reinterpret_cast<char*>(utf8.data()), utf8.size());

    } else if ((file.filetype() == SF7::File::type::non_ascii) && filename.ends_with(".BAS")) {
      auto text = BASIC::detokenise(contents);
      ofs.write(reinterpret_cast<char*>(text.data()), text.size());

    } else
      ofs.write(reinterpret_cast<char*>(contents.data()), contents.size());

    ofs.close();
  }

  return 0;
}
