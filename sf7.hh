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

#include <vector>
#include <string>
#include <stdint.h>

namespace SF7 {

  // Size of each disk (each side is read seperately)
  const int tracks_per_disk = 40;
  const int sectors_per_track = 16;
  const int sector_size = 256;

  // Derived sizes
  const int track_size = sector_size * sectors_per_track;
  const int disk_size = track_size * tracks_per_disk;

  // 'Clusters' is used by the file system
  const int sectors_per_cluster = 4;
  const int cluster_size = sector_size * sectors_per_cluster;

  enum class file_type : uint8_t {
    non_ascii	= 0,
    ascii	= 1,
    hexadecimal = 2,
  };

  class Disk;

  class File {
  private:
    std::string _filename;
    uint8_t _first_cluster;
    file_type _filetype;
    bool _readonly;
    Disk *_disk;

    File(std::string fn, uint8_t fc, uint8_t attr, Disk* d);

    enum _file_attribute_flags : uint8_t {
      FILE_ATTR_RO		= 0x80,
      FILE_ATTR_TYPE_MASK		= 0x0f,
    };

    friend class Disk;

  public:
    File& operator=(const File& other) = default;

    std::string filename(void) const;
    file_type filetype(void) const;
    bool readonly(void) const;

    const std::vector<uint8_t> read(void);
  };

  class Disk {
  private:
    enum class _disk_structure {
      disk_info_start		= 0,
      disk_info_size		= sector_size,

      _reserved_0_start		= sector_size,
      _reserved_0_size		= 15 * sector_size,

      system_programs_start	= track_size,
      system_programs_size	= 18 * track_size,

      directory_start		= 20 * track_size,
      directory_size		= 12 * sector_size,

      FAT_start			= (20 * track_size) + (12 * sector_size),
      FAT_size			= 4 * sector_size,

      user_start		= 21 * track_size,
      user_size			= 19 * track_size,
    };

    struct _dir_entry {
      const char filename[12];	// 8.3, including the period
      uint8_t first_cluster;
      uint8_t attribute;
      uint8_t _unused[2];
    };

    const int dir_entry_size = sizeof(_dir_entry);
    const int max_dir_entries = static_cast<int>(_disk_structure::directory_size) / dir_entry_size;

    enum class _fat_entry_flags {
      LAST_CLUSTER_PREFIX		= 0xc0,
      LAST_CLUSTER_MASK			= 0xf0,
      LAST_CLUSTER_NUM_SECTORS_MASK	= 0x0f,

      RESERVED		= 0xfe,
      UNUSED		= 0xff,
    };

    std::vector<uint8_t> _data;

    // Low-level functions
    void _read_sector(uint16_t snum, std::vector<uint8_t>& dest);
    void _read_cluster(uint8_t cnum, std::vector<uint8_t>& dest);
    uint8_t _read_fat_entry(uint8_t cnum);

    friend class File;

  public:
    Disk();

    void load_data(const uint8_t* data, uint16_t length);

    const std::vector<File> list_directory(void);

  }; // class Disk


}; // namespace sf7
