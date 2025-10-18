/*
  sf7000-fs
  Copyright 2023 Ian Tester

  This library is free software: you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  This library is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this library.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Sega SF-7000 Super Control Station disk/file routines

use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::HashSet;
use std::fmt;

use sc3000_charset::{CharacterSet, SC3000String};

/// Tracks per disk (each side is read seperately)
pub const TRACKS_PER_DISK: usize = 40;
/// Sectors per track
pub const SECTORS_PER_TRACK: usize = 16;
/// Size of each sector
pub const SECTOR_SIZE: usize = 256;

/// Size of each track (derived from SECTOR_SIZE and SECTORS_PER_TRACK)
pub const TRACK_SIZE: usize = SECTOR_SIZE * SECTORS_PER_TRACK;
/// Size of one side of a disk (derived from TRACK_SIZE and TRACKS_PER_DISK)
pub const DISK_SIZE: usize = TRACK_SIZE * TRACKS_PER_DISK;

/// Number of sectors in a 'cluster' used by the file system
pub const SECTORS_PER_CLUSTER: usize = 4;
/// Size of each cluster (derived from SECTOR_SIZE and SECTORS_PER_CLUSTER)
pub const CLUSTER_SIZE: usize = SECTOR_SIZE * SECTORS_PER_CLUSTER;

/// Offset of disk ID start
pub const DISK_ID_START: usize = 0;
/// Offset of disk ID end
pub const DISK_ID_END: usize = 4;

/// Offset of disk name start
pub const DISK_NAME_START: usize = 4;
/// Offset of disk name end
pub const DISK_NAME_END: usize = 32;

/// Offset of initial program loader start
pub const DISK_IPL_START: usize = 32;
/// Offset of initial program loader end (end of first sector)
pub const DISK_IPL_END: usize = SECTOR_SIZE;

/// Offset of reserved area start
pub const DISK_RESERVED_0_START: usize = SECTOR_SIZE;
/// Offset of reserved area end (end of first track)
pub const DISK_RESERVED_0_END: usize = TRACK_SIZE;

/// Offset of system programs start (second track)
pub const DISK_SYSTEM_PROGRAMS_START: usize = TRACK_SIZE;
/// Offset of system programs end (20 tracks later)
pub const DISK_SYSTEM_PROGRAMS_END: usize = 20 * TRACK_SIZE;

/// Offset of directory start (after system programs)
pub const DISK_DIRECTORY_START: usize = DISK_SYSTEM_PROGRAMS_END;
/// Offset of directory end (12 sectors later)
pub const DISK_DIRECTORY_END: usize = DISK_DIRECTORY_START + (12 * SECTOR_SIZE);

/// Offset of file allocation table start (after directory)
pub const DISK_FAT_START: usize = DISK_DIRECTORY_END;
/// Offset of file allocation table end
pub const DISK_FAT_END: usize = 21 * TRACK_SIZE;

/// Offset of user data start (after FAT)
pub const DISK_USER_START: usize = DISK_FAT_END;
/// Offset of user data end (end of disk)
pub const DISK_USER_END: usize = DISK_SIZE;

/// File types as stored on disk
#[derive(Copy, Clone, PartialEq, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum FileType {
    /// Non-ASCII file (probably tokenised BASIC)
    NonAscii = 0,

    /// ASCII file
    Ascii = 1,

    /// Hexadecimal (probably raw data)
    Hexadecimal = 2,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileType::NonAscii => write!(f, "non-ASCII"),
            FileType::Ascii => write!(f, "ASCII"),
            FileType::Hexadecimal => write!(f, "hexadecimal"),
        }
    }
}

const FILE_ATTR_TYPE_MASK: u8 = 0x0f;
const FILE_ATTR_RO: u8 = 0x80;

/// Size of each directory entry
pub const DIR_ENTRY_SIZE: usize = 16;
/// Maxinum number of directory entries
pub const MAX_DIR_ENTRIES: usize = (12 * SECTOR_SIZE) / DIR_ENTRY_SIZE;

const FAT_LAST_CLUSTER_MASK: u8 = 0xf0;
const FAT_LAST_CLUSTER_PREFIX: u8 = 0xc0;
const FAT_LAST_CLUSTER_NUM_SECTORS_MASK: u8 = 0x0f;
const FAT_RESERVED: u8 = 0xfe;
const FAT_UNUSED: u8 = 0xff;

/// SF-7000 disk image
#[derive(Clone, Default, Debug)]
pub struct Disk {
    data: Vec<u8>,
}

impl Disk {
    /// Constructor
    pub fn new() -> Self {
        Disk {
            data: Vec::with_capacity(DISK_SIZE),
        }
    }

    /// Load data into the disk image
    pub fn load_data(&mut self, newdata: &[u8]) {
        self.data.append(&mut newdata.to_vec());
    }

    /// Is this a system disk?
    pub fn is_sys(&self) -> bool {
        &self.data[DISK_ID_START..DISK_ID_END] == b"SYS:"
    }

    /// Disk name
    pub fn name(&self, cset: CharacterSet) -> SC3000String {
        SC3000String::from_bytes(&self.data[DISK_NAME_START..DISK_NAME_END], cset)
    }

    /// Initial Program Loader
    pub fn ipl(&self) -> &[u8] {
        &self.data[DISK_IPL_START..DISK_IPL_END]
    }

    /// List the files on disk
    pub fn list_directory(&self, cset: CharacterSet) -> Vec<File> {
        let mut files: Vec<File> = Vec::new();

        for i in 0..MAX_DIR_ENTRIES {
            let entry_start = DISK_DIRECTORY_START + (i * DIR_ENTRY_SIZE);
            if self.data[entry_start..entry_start + 16] == [0_u8; 16] {
                continue;
            }

            let name = self.data[entry_start..entry_start + 12].to_vec();

            let (mut name_part, mut ext_part) = name.split_at(8);
            while name_part.ends_with(b" ") {
                name_part = name_part.strip_suffix(b" ").unwrap();
            }
            while ext_part.ends_with(b" ") {
                ext_part = ext_part.strip_suffix(b" ").unwrap();
            }

            let mut name = name_part.to_vec();
            name.extend(ext_part);
            name = name
                .iter()
                .flat_map(|b| {
                    if *b == b'/' {
                        vec![b'-', b'-']
                    } else {
                        vec![*b]
                    }
                })
                .collect();

            let first_cluster = self.data[entry_start + 12];
            let attr = self.data[entry_start + 13];

            files.push(File {
                name: SC3000String::from_bytes(&name, cset),
                first_cluster,
                file_type: (attr & FILE_ATTR_TYPE_MASK).try_into().unwrap(),
                readonly: attr & FILE_ATTR_RO != 0,
            });
        }

        files
    }

    fn read_sector(&self, sector_num: u16) -> Vec<u8> {
        let i = (sector_num as usize) * SECTOR_SIZE;
        self.data[i..i + SECTOR_SIZE].to_vec()
    }

    fn read_cluster(&self, cluster_num: u8) -> Vec<u8> {
        let mut dest = Vec::with_capacity(CLUSTER_SIZE);

        for s in 0..SECTORS_PER_CLUSTER {
            dest.append(
                &mut self.read_sector((((cluster_num as usize) * SECTORS_PER_CLUSTER) + s) as u16),
            );
        }

        dest
    }

    fn fat_entry(&self, cluster_num: u8) -> u8 {
        self.data[DISK_FAT_START + cluster_num as usize]
    }

    /// Read file contents
    pub fn read_file(&self, file: &File) -> Vec<u8> {
        let mut contents = Vec::new();
        let mut visited_clusters = HashSet::new();

        let mut cluster_num = file.first_cluster;
        while cluster_num < 160 {
            if visited_clusters.contains(&cluster_num) {
                eprintln!("FAT loop detected when reading file \"{}\".", file.name);
                break;
            }
            visited_clusters.insert(cluster_num);

            let fat_entry = self.fat_entry(cluster_num);
            if fat_entry & FAT_LAST_CLUSTER_MASK == FAT_LAST_CLUSTER_PREFIX {
                let num_sectors = fat_entry & FAT_LAST_CLUSTER_NUM_SECTORS_MASK;
                let sector_num_start = (cluster_num as u16) * (SECTORS_PER_CLUSTER as u16);
                for i in 0..num_sectors {
                    contents.append(&mut self.read_sector(sector_num_start + i as u16));
                }
                break;
            }

            contents.append(&mut self.read_cluster(cluster_num));

            cluster_num = fat_entry;
        }

        contents
    }
}

/// SF-7000 file
#[derive(Clone, Debug)]
pub struct File {
    /// File name
    pub name: SC3000String,

    /// Index of first cluster
    first_cluster: u8,

    /// File type
    pub file_type: FileType,

    /// Is the file read-only?
    pub readonly: bool,
}
