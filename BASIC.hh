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
#include <unordered_map>
#include <vector>

namespace BASIC {

  extern std::unordered_map<uint8_t, std::string> tokens, funcs;

  std::vector<uint8_t> detokenise(const std::vector<uint8_t> bytes, const std::unordered_map<uint8_t, std::string>& charmap);

}; // namespace BASIC
