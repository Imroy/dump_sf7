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
#include <iostream>
#include <fstream>

int main(int argc, char* argv[]) {
  if (argc < 2)
    return -1;

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

  auto files = disk.list_directory();
  for (auto file : files) {
    std::cout << file.filename();
    switch (file.filetype()) {
    case SF7::file_type::non_ascii:
      std::cout << "\tnon-ASCII";
      break;

    case SF7::file_type::ascii:
      std::cout << "\tASCII";
      break;

    case SF7::file_type::hexadecimal:
      std::cout << "\thexadecimal";
      break;
    }

    if (file.readonly())
      std::cout << "\tread-only";
    else
      std::cout << "\tread-write";
    std::cout << std::endl;

    auto contents = file.read();
    std::ofstream ofs;
    ofs.open(file.filename(), std::ios_base::out);
    ofs.write(reinterpret_cast<char*>(contents.data()), contents.size());
    ofs.close();
  }

  return 0;
}
