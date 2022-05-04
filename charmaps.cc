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
#include "charmaps.hh"

void append_string_to_bytes(std::vector<uint8_t>& bytes, const std::string& str) {
  for (auto b : str)
    bytes.push_back(b);
}

std::vector<uint8_t> convert_utf8(const std::vector<uint8_t>& source, std::unordered_map<uint8_t, std::string>& charmap) {
  std::vector<uint8_t> dest;
  dest.reserve(source.size() * 3);

  for (auto byte : source) {
    if (charmap.find(byte) != charmap.end()) {
      append_string_to_bytes(dest, charmap[byte]);
      continue;
    }

    dest.push_back(byte);
  }

  dest.shrink_to_fit();
  return dest;
}

namespace Sega {

  namespace SC3000 {

    // https://en.wikipedia.org/wiki/Sega_SC-3000_character_set
    // https://www.smspower.org/Development/SC-3000Font
    std::unordered_map<uint8_t, std::string> japan_charmap = {
      { 0x0d, "\x0a" },

      { 0x5c, "\u00A5" }, { 0x5f, "\u03C0" },

      { 0x80, "\u2502" }, { 0x81, "\u2500" }, { 0x82, "\u2534" }, { 0x83, "\u252c" },
      { 0x84, "\u2524" }, { 0x85, "\u251c" }, { 0x86, "\u250c" }, { 0x87, "\u2514" },
      { 0x88, "\u2510" }, { 0x89, "\u2518" }, { 0x8a, "\u256d" }, { 0x8b, "\u2570" },
      { 0x8c, "\u256e" }, { 0x8d, "\u256f" }, { 0x8e, "\u2191" }, { 0x8f, "\u2190" },

      { 0x90, "\u2592" }, { 0x91, "\u2573" }, { 0x92, "\u253c" }, { 0x93, "\u2571" },
      { 0x94, "\u2572" }, { 0x95, "\u25e2" }, { 0x96, "\u25e3" }, { 0x97, "\u25e5" },
      { 0x98, "\u25e4" }, { 0x99, "\u2581" }, { 0x9a, "\u2582" }, { 0x9b, "\u2584" },
      { 0x9c, "\u2580" }, { 0x9d, "\U0001fb82" }, { 0x9e, "\u2594" }, { 0x9f, "\u258f" },

      { 0xa0, "\u00a0" }, { 0xa1, "\u3002" }, { 0xa2, "\u300c" }, { 0xa3, "\u300d" },
      { 0xa4, "\u3001" }, { 0xa5, "\u30fb" }, { 0xa6, "\u30f2" }, { 0xa7, "\u30a1" },
      { 0xa8, "\u30a3" }, { 0xa9, "\u30a5" }, { 0xaa, "\u30a7" }, { 0xab, "\u30a9" },
      { 0xac, "\u30e3" }, { 0xad, "\u30e5" }, { 0xae, "\u30e7" }, { 0xaf, "\u30c3" },

      { 0xb0, "\u30fc" }, { 0xb1, "\u30a2" }, { 0xb2, "\u30a4" }, { 0xb3, "\u30a6" },
      { 0xb4, "\u30a8" }, { 0xb5, "\u30aa" }, { 0xb6, "\u30ab" }, { 0xb7, "\u30ad" },
      { 0xb8, "\u30af" }, { 0xb9, "\u30b1" }, { 0xba, "\u30b3" }, { 0xbb, "\u30b5" },
      { 0xbc, "\u30b7" }, { 0xbd, "\u30b9" }, { 0xbe, "\u30bb" }, { 0xbf, "\u30bd" },

      { 0xc0, "\u30bf" }, { 0xc1, "\u30c1" }, { 0xc2, "\u30c4" }, { 0xc3, "\u30c6" },
      { 0xc4, "\u30c8" }, { 0xc5, "\u30ca" }, { 0xc6, "\u30cb" }, { 0xc7, "\u30cc" },
      { 0xc8, "\u30cd" }, { 0xc9, "\u30ce" }, { 0xca, "\u30cf" }, { 0xcb, "\u30d2" },
      { 0xcc, "\u30d5" }, { 0xcd, "\u30d8" }, { 0xce, "\u30db" }, { 0xcf, "\u30de" },

      { 0xd0, "\u30df" }, { 0xd1, "\u30e0" }, { 0xd2, "\u30e1" }, { 0xd3, "\u30e2" },
      { 0xd4, "\u30e4" }, { 0xd5, "\u30e6" }, { 0xd6, "\u30e8" }, { 0xd7, "\u30e9" },
      { 0xd8, "\u30ea" }, { 0xd9, "\u30eb" }, { 0xda, "\u30ec" }, { 0xdb, "\u30ed" },
      { 0xdc, "\u30ef" }, { 0xdd, "\u30f3" }, { 0xde, "\u309b" }, { 0xdf, "\u309c" },

      { 0xe0, "\u258e" }, { 0xe1, "\u258c" }, { 0xe2, "\u2590" }, { 0xe3, "\U0001fb87" },
      { 0xe4, "\u2595" }, { 0xe5, "\u2588" }, { 0xe6, "\ufffd" }, { 0xe7, "\ufffd" },
      { 0xe8, "\ufffd" }, { 0xe9, "\ufffd" }, { 0xea, "\u259e" }, { 0xeb, "\u25cb" },
      { 0xec, "\u25cf" }, { 0xed, "\u5e74" }, { 0xee, "\u6708" }, { 0xef, "\u65e5" },

      { 0xf0, "\u706b" }, { 0xf1, "\u6c34" }, { 0xf2, "\u6728" }, { 0xf3, "\u91d1" },
      { 0xf4, "\u571f" }, { 0xf5, "\u1660" }, { 0xf6, "\u2665" }, { 0xf7, "\u2666" },
      { 0xf8, "\u2663" }, { 0xf9, "\ufffd" }, { 0xfa, "\ufffd" }, { 0xfb, "\ufffd" },
      { 0xfc, "\ufffd" }, { 0xfd, "\U0001fbc5" }, { 0xfe, "\u00f7" },
    };

    std::unordered_map<uint8_t, std::string> export_charmap = {
      { 0x0d, "\x0a" },

      { 0x5c, "\u00A5" }, { 0x5f, "\u03C0" },

      { 0x80, "\u2502" }, { 0x81, "\u2500" }, { 0x82, "\u2534" }, { 0x83, "\u252c" },
      { 0x84, "\u2524" }, { 0x85, "\u251c" }, { 0x86, "\u250c" }, { 0x87, "\u2514" },
      { 0x88, "\u2510" }, { 0x89, "\u2518" }, { 0x8a, "\u256d" }, { 0x8b, "\u2570" },
      { 0x8c, "\u256e" }, { 0x8d, "\u256f" }, { 0x8e, "\u2191" }, { 0x8f, "\u2190" },

      { 0x90, "\u2592" }, { 0x91, "\u2573" }, { 0x92, "\u253c" }, { 0x93, "\u2571" },
      { 0x94, "\u2572" }, { 0x95, "\u25e2" }, { 0x96, "\u25e3" }, { 0x97, "\u25e5" },
      { 0x98, "\u25e4" }, { 0x99, "\u2581" }, { 0x9a, "\u2582" }, { 0x9b, "\u2584" },
      { 0x9c, "\u2580" }, { 0x9d, "\U0001fb82" }, { 0x9e, "\u2594" }, { 0x9f, "\u258f" },

      { 0xa0, "\u00C2" }, { 0xa1, "\u01CD" }, { 0xa2, "\u00C1" }, { 0xa3, "\u00C0" },
      { 0xa4, "\u00C4" }, { 0xa5, "\u00C5" }, { 0xa6, "\u00C3" }, { 0xa7, "\u0100" },
      { 0xa8, "\u00CA" }, { 0xa9, "\u011A" }, { 0xaa, "\u00CB" }, { 0xab, "\u0112" },
      { 0xac, "\u00C9" }, { 0xad, "\u00C8" }, { 0xae, "\u00D1" }, { 0xaf, "\u004E" },

      { 0xb0, "\u01cf" }, { 0xb1, "\u00cc" }, { 0xb2, "\u00cd" }, { 0xb3, "\u00cf" },
      { 0xb4, "\u00ce" }, { 0xb5, "\u012a" }, { 0xb6, "\u00d4" }, { 0xb7, "\u01d1" },
      { 0xb8, "\u004f" }, { 0xb9, "\u00d3" }, { 0xba, "\u00d2" }, { 0xbb, "\u00d6" },
      { 0xbc, "\u00d5" }, { 0xbd, "\u00d8" }, { 0xbe, "\u01d3" }, { 0xbf, "\u00da" },

      { 0xc0, "\u00d9" }, { 0xc1, "\u00dc" }, { 0xc2, "\u016a" }, { 0xc3, "\u03b1" },
      { 0xc4, "\u03b2" }, { 0xc5, "\u03b8" }, { 0xc6, "\u03bb" }, { 0xc7, "\u03bc" },
      { 0xc8, "\u03a3" }, { 0xc9, "\u03a6" }, { 0xca, "\u03a9" }, { 0xcb, "\u00c7" },
      { 0xcc, "\u00bf" }, { 0xcd, "\u00a1" }, { 0xce, "\ufffd" }, { 0xcf, "\u00a3" },

      { 0xe0, "\u258e" }, { 0xe1, "\u258c" }, { 0xe2, "\u2590" }, { 0xe3, "\U0001fb87" },
      { 0xe4, "\u2595" }, { 0xe5, "\u2588" }, { 0xe6, "\ufffd" }, { 0xe7, "\ufffd" },
      { 0xe8, "\ufffd" }, { 0xe9, "\ufffd" }, { 0xea, "\u259e" }, { 0xeb, "\u25cb" },
      { 0xec, "\u25cf" },

      { 0xf5, "\u2660" }, { 0xf6, "\u2665" }, { 0xf7, "\u2666" }, { 0xf8, "\u2663" },
      { 0xf9, "\ufffd" }, { 0xfa, "\ufffd" }, { 0xfb, "\ufffd" }, { 0xfc, "\ufffd" },
      { 0xfd, "\U0001fbc5" }, { 0xfe, "\u00f7" },
    };

  }; // namespace SC3000

}; // namespace Sega
