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

namespace SF7 {

  const std::vector<dir_entry> list_directory(const std::vector<uint8_t>& disk) {
    std::vector<dir_entry> entries;
    entries.reserve(max_dir_entries);

    auto raw_entries = reinterpret_cast<_raw_dir_entry*>(const_cast<unsigned char*>(disk.data()) + static_cast<int>(disk_structure::directory_start));
    for (int i = 0; i < max_dir_entries; i++) {
      auto raw_entry = raw_entries[i];
      if (raw_entry.filename[0] == 0)
	continue;

      dir_entry entry;
      entry.filename = std::string(raw_entry.filename, 12);
      entry.first_cluster = raw_entry.first_cluster;
      entry.filetype = static_cast<file_type>(raw_entry.attribute & FILE_ATTR_TYPE_MASK);
      entry.readonly = raw_entry.attribute & FILE_ATTR_RO;
      entries.push_back(entry);
    }

    return entries;
  }

  const std::vector<uint8_t> read_file(const std::vector<uint8_t>& disk, const dir_entry& file) {
    std::vector<uint8_t> bytes;

    uint8_t cnum = file.first_cluster;
    while (cnum < 160) {
      auto fat_entry = _read_fat_entry(disk, cnum);
      if ((fat_entry & static_cast<uint8_t>(_fat_entry_flags::LAST_CLUSTER_MASK)) == static_cast<uint8_t>(_fat_entry_flags::LAST_CLUSTER_PREFIX)) {
	auto sectors = fat_entry & static_cast<uint8_t>(_fat_entry_flags::LAST_CLUSTER_NUM_SECTORS_MASK);
	uint16_t snum_start = cnum * sectors_per_cluster;
	for (uint16_t i = 0; i < sectors; i++)
	  _read_sector(disk, snum_start + i, bytes);
      } else
	_read_cluster(disk, cnum, bytes);

      cnum = fat_entry;
    }

    return bytes;
  }

  void _read_sector(const std::vector<uint8_t>& disk, uint16_t snum, std::vector<uint8_t>& dest) {
    dest.reserve(dest.size() + sector_size);

    uint16_t bnum_start = snum * sector_size;
    for (uint16_t b = 0; b < sector_size; b++)
      dest.push_back(disk[bnum_start + b]);
  }

  void _read_cluster(const std::vector<uint8_t>& disk, uint8_t cnum, std::vector<uint8_t>& dest) {
    dest.reserve(dest.size() + cluster_size);

    uint16_t snum_start = cnum * sectors_per_cluster;
    for (uint8_t s = 0; s < sectors_per_cluster; s++)
      _read_sector(disk, snum_start + s, dest);
  }

  uint8_t _read_fat_entry(const std::vector<uint8_t>& disk, uint8_t cnum) {
    return disk[static_cast<int>(disk_structure::FAT_start) + cnum];
  }


}; // namespace sf7
