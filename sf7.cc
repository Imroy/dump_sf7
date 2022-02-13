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
#include <string.h>
#include <unordered_set>

namespace SF7 {

  File::File(std::string rfn, std::string fn, uint8_t fc, uint8_t attr, const Disk* d) :
    _raw_filename(rfn),
    _filename(fn),
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

  std::string File::raw_filename(void) const {
    return _raw_filename;
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
    std::unordered_set<uint8_t> visited_clusters;

    uint8_t cnum = _first_cluster;
    while (cnum < 160) {
      if (visited_clusters.find(cnum) != visited_clusters.end()) {
	std::cerr << "FAT loop detected when reading file \"" << _filename << "\"." << std::endl;
	return bytes;
      }
      visited_clusters.insert(cnum);

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
    auto i = static_cast<int>(_structure::id_start);
    return (_data[i] == 'S')
      && (_data[i + 1] == 'Y')
      && (_data[i + 2] == 'S')
      && (_data[i + 3] == ':');
  }

  const std::string Disk::name(void) const {
    return std::string(reinterpret_cast<char*>(const_cast<uint8_t*>(_data.data())) + static_cast<int>(_structure::name_start),
		       static_cast<int>(_structure::name_size));
  }

  const std::vector<uint8_t> Disk::IPL(void) const {
    std::vector<uint8_t> bytes(static_cast<int>(_structure::IPL_size));
    memcpy(bytes.data(),
	   reinterpret_cast<char*>(const_cast<uint8_t*>(_data.data())) + static_cast<int>(_structure::IPL_start),
	   static_cast<int>(_structure::IPL_size));

    return bytes;
  }

  const std::vector<File> Disk::list_directory(std::unordered_map<uint8_t, std::string>& charmap) const {
    std::vector<File> files;
    files.reserve(max_dir_entries);

    auto entries = reinterpret_cast<_dir_entry*>(const_cast<unsigned char*>(_data.data()) + static_cast<int>(_structure::directory_start));
    for (int i = 0; i < max_dir_entries; i++) {
      auto entry = entries[i];
      if (entry.filename[0] == 0)
	continue;

      // Convert Sega 'ASCII' into UTF-8 text
      std::vector<uint8_t> sega_filename(12);
      memcpy(sega_filename.data(), entry.filename, 12);
      auto utf8_filename = Sega::convert_utf8(sega_filename, charmap);
      std::string filename(reinterpret_cast<char*>(utf8_filename.data()), utf8_filename.size());

      // Remove spaces at end of the two parts of the filename
      filename = filename.substr(0, filename.find_last_not_of(" ", 7) + 1)
	+ filename.substr(8, filename.find_last_not_of(" ", 11) - 7);

      // Replace slashes (/) with a double dash (--)
      for (std::string::size_type pos{}, count{};
	   filename.npos != (pos = filename.find("/", pos, 1));
	   pos++, ++count) {
	filename.replace(pos, 1, "--", 2);
      }

      File file(std::string(entry.filename, 12),
		filename,
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
