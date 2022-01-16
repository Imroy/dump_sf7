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
#pragma once

#include <string>
#include <vector>
#include <map>
#include <stdint.h>

// Utility function
void append_string_to_bytes(std::vector<uint8_t>& bytes, const std::string& str);

namespace Sega {

  //! Convert 8-bit text to UTF-8 encoded text using the Japanese SC-3000 character set
  std::vector<uint8_t> convert_utf8_japan(const std::vector<uint8_t>& source);

  //! Convert 8-bit text to UTF-8 encoded text using the 'export' SC-3000 character set
  std::vector<uint8_t> convert_utf8_export(const std::vector<uint8_t>& source);

  extern std::map<uint8_t, std::string> japan_charmap;
  extern std::map<uint8_t, std::string> export_charmap;

}; // namespace Sega
