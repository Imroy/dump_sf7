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
#include "BASIC.hh"
#include "charmaps.hh"

#include <map>
#include <vector>
#include <stdint.h>

namespace BASIC {

  // Extracted from hex dumps of Sega SC-3000 BASIC Level 3 (cartridge)
  // and Sega SC-3000 Disk BASIC v1.1p (floppy)

  std::map<uint8_t, std::string> tokens = {
    { 0x81, "INPUT$" }, { 0x82, "LIST" }, { 0x83, "LLIST" },
    { 0x84, "AUTO" }, { 0x85, "DELETE" }, { 0x86, "RUN" }, { 0x87, "CONT" },
    { 0x88, "LOAD" }, { 0x89, "SAVE" }, { 0x8a, "VERIFY" }, { 0x8b, "NEW" },
    { 0x8c, "RENUM" }, { 0x8d, "FILES" }, { 0x8e, "LFILES" }, { 0x8f, "BOOT" },

    { 0x90, "REM" }, { 0x91, "PRINT" }, { 0x92, "LPRINT" }, { 0x93, "DATA" },
    { 0x94, "DEF" }, { 0x95, "INPUT" }, { 0x96, "READ" }, { 0x97, "STOP" },
    { 0x98, "END" }, { 0x99, "LET" }, { 0x9a, "DIM" }, { 0x9b, "FOR" },
    { 0x9c, "NEXT" }, { 0x9d, "GOTO" }, { 0x9e, "GOSUB" }, { 0x9f, "GO" },

    { 0xa0, "ON" }, { 0xa1, "RETURN" }, { 0xa2, "ERASE" }, { 0xa3, "CURSOR" },
    { 0xa4, "IF" }, { 0xa5, "RESTORE" }, { 0xa6, "SCREEN" }, { 0xa7, "COLOR" },
    { 0xa8, "LINE" }, { 0xa9, "SOUND" }, { 0xaa, "BEEP" }, { 0xab, "CONSOLE" },
    { 0xac, "CLS" }, { 0xad, "OUT" }, { 0xae, "CALL" }, { 0xaf, "POKE" },

    { 0xb0, "PSET" }, { 0xb1, "PRESET" }, { 0xb2, "PAINT" }, { 0xb3, "BLINE" },
    { 0xb4, "POSITION" }, { 0xb5, "HCOPY" }, { 0xb6, "SPRITE" }, { 0xb7, "PATTERN" },
    { 0xb8, "CIRCLE" }, { 0xb9, "BCIRCLE" }, { 0xba, "MAG" }, { 0xbb, "VPOKE" },
    { 0xbc, "MOTOR" }, { 0xbd, "OPEN" }, { 0xbe, "CLOSE" }, { 0xbf, "COMSET" },

    { 0xc0, "^" }, { 0xc1, "*" }, { 0xc2, "/" }, { 0xc3, "MOD" },
    { 0xc4, "+" }, { 0xc5, "-" }, { 0xc6, "<>" }, { 0xc7, ">=" },
    { 0xc8, "<=" }, { 0xc9, ">" }, { 0xca, "<" }, { 0xcb, "=" },
    { 0xcc, "NOT" }, { 0xcd, "AND" }, { 0xce, "OR" }, { 0xcf, "XOR" },

    { 0xd0, "CLOADM" }, { 0xd1, "CSAVEM" }, { 0xd2, "VERIFYM" },

    { 0xe0, "FN" }, { 0xe1, "TO" }, { 0xe2, "STEP" }, { 0xe3, "THEN" },
    { 0xe4, "TAB" }, { 0xe5, "SPC" }, { 0xe7, "OUTPUT" },
  };

  std::map<uint8_t, std::string> funcs = {
    { 0x80, "ABS" }, { 0x81, "RND" }, { 0x82, "SIN" }, { 0x83, "COS" },
    { 0x84, "TAN" }, { 0x85, "ASN" }, { 0x86, "ACS" }, { 0x87, "ATN" },
    { 0x88, "LOG" }, { 0x89, "LGT" }, { 0x8a, "LTW" }, { 0x8b, "EXP" },
    { 0x8c, "RAD" }, { 0x8d, "DEG" }, { 0x8e, "PI" }, { 0x8f, "SQR" },

    { 0x90, "INT" }, { 0x91, "SGN" }, { 0x92, "ASC" }, { 0x93, "LEN" },
    { 0x94, "VAL" }, { 0x95, "PEEK" }, { 0x96, "INP" }, { 0x97, "FRE" },
    { 0x98, "VPEEK" }, { 0x99, "STICK" }, { 0x9a, "STRIG" }, { 0x9b, "EOF" },
    { 0x9c, "LOC" }, { 0x9d, "LOD" }, { 0x9e, "DSKF" },

    { 0xa0, "CHR$" }, { 0xa1, "HEX$" }, { 0xa2, "INKEY$" }, { 0xa3, "LEFT$" },
    { 0xa4, "RIGHT$" }, { 0xa5, "MID$" }, { 0xa6, "STR$" }, { 0xa7, "TIME$" },
  };

  void _append_hex(std::vector<uint8_t>& bytes, uint8_t nibble) {
    if (nibble < 10)
      bytes.push_back(0x30 + nibble);
    else
      bytes.push_back(0x61 + nibble - 10);
  }

  std::vector<uint8_t> detokenise(const std::vector<uint8_t> bytes) {
    std::vector<uint8_t> output;
    output.reserve(bytes.size() * 10);

    bool start_of_line = true, use_funcs, is_text;
    for (auto bi = bytes.begin(); bi != bytes.end(); bi++) {
      if (start_of_line) {
	// Eat five bytes of binary data at the start of each line
	bi++;
	if (bi == bytes.end())
	  break;
	// Second and third bytes are the line number
	int lineno = *bi;
	bi++;
	if (bi == bytes.end())
	  break;

	lineno |= (*bi) << 8;
	bi++;
	if (bi == bytes.end())
	  break;
	bi++;
	if (bi == bytes.end())
	  break;
	append_string_to_bytes(output, std::to_string(lineno) + " ");

	start_of_line = false;
	use_funcs = false;
	is_text = false;
	continue;
      }

      // Ordinary ASCII characters
      if ((*bi >= 32) && (*bi < 127)) {
	output.push_back(*bi);
	if (*bi == ':')
	  use_funcs = false;
	else if (*bi == '"')
	  is_text ^= true;
	continue;
      }

      // Newline
      if (*bi == 0x0d) {
	output.push_back('\x0a');
	start_of_line = true;
	is_text = false;
	continue;
      }

      if (is_text) {
	if (Sega::export_charmap.contains(*bi)) {
	  append_string_to_bytes(output, Sega::export_charmap[*bi]);
	  continue;
	}
	output.push_back(*bi);
	continue;
      }

      if (use_funcs) {
	if (funcs.contains(*bi)) {
	  append_string_to_bytes(output, funcs[*bi]);
	  continue;
	}
      }
      if (tokens.contains(*bi)) {
	append_string_to_bytes(output, tokens[*bi]);

	// REM
	if (*bi == 0x90)
	  is_text = true;

	continue;
      }
      use_funcs = true;

      // ?
   }

    output.shrink_to_fit();
    return output;
  }

}; // namespace BASIC
