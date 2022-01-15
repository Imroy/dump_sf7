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
#include <string.h>

namespace SF7 {

  File::File(std::string fn, uint8_t fc, uint8_t attr, const Disk* d) :
    _filename(std::move(fn)),
    _first_cluster(fc),
    _filetype(static_cast<File::type>(attr & FILE_ATTR_TYPE_MASK)),
    _readonly(attr & FILE_ATTR_RO),
    _disk(d)
  {}

  void File::_read_cluster(uint8_t cnum, std::vector<uint8_t>& dest) const {
    dest.reserve(dest.size() + cluster_size);

    uint16_t snum_start = cnum * sectors_per_cluster;
    for (uint8_t s = 0; s < sectors_per_cluster; s++)
      _disk->_read_sector(snum_start + s, dest);
  }

  uint8_t File::_fat_entry(uint8_t cnum) const {
    return _disk->_data[static_cast<int>(_disk->_structure::FAT_start) + cnum];
  }

  std::string File::filename(void) const {
    return _filename;
  }

  File::type File::filetype(void) const {
    return _filetype;
  }

  bool File::readonly(void) const {
    return _readonly;
  }

  const std::vector<uint8_t> File::read(void) {
    std::vector<uint8_t> bytes;

    uint8_t cnum = _first_cluster;
    while (cnum < 160) {
      auto fat_entry = _fat_entry(cnum);
      if ((fat_entry & static_cast<uint8_t>(Disk::_fat_entry_flags::LAST_CLUSTER_MASK)) == static_cast<uint8_t>(Disk::_fat_entry_flags::LAST_CLUSTER_PREFIX)) {
	auto sectors = fat_entry & static_cast<uint8_t>(Disk::_fat_entry_flags::LAST_CLUSTER_NUM_SECTORS_MASK);
	uint16_t snum_start = cnum * sectors_per_cluster;
	for (uint16_t i = 0; i < sectors; i++)
	  _disk->_read_sector(snum_start + i, bytes);
      } else
	_read_cluster(cnum, bytes);

      cnum = fat_entry;
    }

    return bytes;
  }


  Disk::Disk() {
    _data.reserve(disk_size);
  }

  void Disk::load_data(const uint8_t* data, uint16_t length) {
    auto end = _data.size();
    _data.resize(_data.size() + length);
    memcpy(_data.data() + end, data, length);
  }

  bool Disk::is_sys(void) const {
    return (_data[0] == 'S')
      && (_data[1] == 'Y')
      && (_data[2] == 'S')
      && (_data[3] == ':');
  }

  const std::string Disk::name(void) const {
    return std::string(reinterpret_cast<char*>(const_cast<uint8_t*>(_data.data())) + 4, 28);
  }

  const std::vector<File> Disk::list_directory() const {
    std::vector<File> files;
    files.reserve(max_dir_entries);

    auto entries = reinterpret_cast<_dir_entry*>(const_cast<unsigned char*>(_data.data()) + static_cast<int>(_structure::directory_start));
    for (int i = 0; i < max_dir_entries; i++) {
      auto entry = entries[i];
      if (entry.filename[0] == 0)
	continue;

      File file(std::string(entry.filename, 12),
		entry.first_cluster,
		entry.attribute,
		this);
      files.push_back(file);
    }

    return files;
  }

  void Disk::_read_sector(uint16_t snum, std::vector<uint8_t>& dest) const {
    dest.reserve(dest.size() + sector_size);

    uint16_t bnum_start = snum * sector_size;
    for (uint16_t b = 0; b < sector_size; b++)
      dest.push_back(_data[bnum_start + b]);
  }


}; // namespace sf7
