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

use std::fmt;
use std::collections::HashSet;
use num_enum::{IntoPrimitive, TryFromPrimitive};

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
pub const DISK_ID_START: usize		= 0;
/// Offset of disk ID end
pub const DISK_ID_END: usize		= 4;

/// Offset of disk name start
pub const DISK_NAME_START: usize	= 4;
/// Offset of disk name end
pub const DISK_NAME_END: usize		= 32;

/// Offset of initial program loader start
pub const DISK_IPL_START: usize		= 32;
/// Offset of initial program loader end (end of first sector)
pub const DISK_IPL_END: usize		= SECTOR_SIZE;

/// Offset of reserved area start
pub const DISK_RESERVED_0_START: usize	= SECTOR_SIZE;
/// Offset of reserved area end (end of first track)
pub const DISK_RESERVED_0_END: usize	= TRACK_SIZE;

/// Offset of system programs start (second track)
pub const DISK_SYSTEM_PROGRAMS_START: usize = TRACK_SIZE;
/// Offset of system programs end (20 tracks later)
pub const DISK_SYSTEM_PROGRAMS_END: usize = 20 * TRACK_SIZE;

/// Offset of directory start (after system programs)
pub const DISK_DIRECTORY_START: usize	= DISK_SYSTEM_PROGRAMS_END;
/// Offset of directory end (12 sectors later)
pub const DISK_DIRECTORY_END: usize	= DISK_DIRECTORY_START + (12 * SECTOR_SIZE);

/// Offset of file allocation table start (after directory)
pub const DISK_FAT_START: usize		= DISK_DIRECTORY_END;
/// Offset of file allocation table end
pub const DISK_FAT_END: usize		= 21 * TRACK_SIZE;

/// Offset of user data start (after FAT)
pub const DISK_USER_START: usize	= DISK_FAT_END;
/// Offset of user data end (end of disk)
pub const DISK_USER_END: usize		= DISK_SIZE;


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


const FILE_ATTR_TYPE_MASK: u8	= 0x0f;
const FILE_ATTR_RO: u8		= 0x80;

/// Size of each directory entry
pub const DIR_ENTRY_SIZE: usize = 16;
/// Maxinum number of directory entries
pub const MAX_DIR_ENTRIES: usize = (12 * SECTOR_SIZE) / DIR_ENTRY_SIZE;

/// SF-7000 disk image
#[derive(Clone, Default, Debug)]
pub struct Disk {
    data: Vec<u8>,

}

impl Disk {
    /// Constructor
    pub fn new() -> Disk {
        Disk { data: Vec::with_capacity(DISK_SIZE) }
    }

    /// Load data into the disk image
    pub fn load_data(&mut self, newdata: &[u8]) {
        self.data.append(&mut newdata.to_vec());
    }

    /// Is this a system disk?
    pub fn is_sys(&self) -> bool {
        &self.data[DISK_ID_START..DISK_ID_START + 4] == b"SYS:"
    }

    /// Disk name
    pub fn name<F>(&self, to_unicode: F) -> String
    where
        F: Fn(&[u8]) -> String
    {
        to_unicode(&self.data[DISK_NAME_START..DISK_NAME_END])
    }

    /// Initial Program Loader
    pub fn ipl(&self) -> &[u8] {
        &self.data[DISK_IPL_START..DISK_IPL_END]
    }

    /// List the files on disk
    pub fn list_directory<F>(&self, to_unicode: F) -> Vec<File>
    where
        F: Fn(&[u8]) -> String
    {
        let mut files: Vec<File> = Vec::new();

        for i in 0..MAX_DIR_ENTRIES {
            let entry_start = DISK_DIRECTORY_START + (i * DIR_ENTRY_SIZE);
            let name = &self.data[entry_start..entry_start + 12];
            if name[0] == b'\0' {
                continue;
            }

            let mut utf8_filename = to_unicode(name);
            let (mut name_part, mut ext_part) = utf8_filename.split_at(8);
            while name_part.ends_with(' ') {
                name_part = name_part.strip_suffix(' ').unwrap();
            }
            while ext_part.ends_with(' ') {
                ext_part = ext_part.strip_suffix(' ').unwrap();
            }
            utf8_filename = name_part.to_owned() + ext_part;
            utf8_filename = utf8_filename.replace('/', "--");

            files.push(File {
                name: utf8_filename,
                first_cluster: self.data[entry_start + 12],
                file_type: (self.data[entry_start + 13] & FILE_ATTR_TYPE_MASK).try_into().unwrap(),
                readonly: self.data[entry_start + 13] & FILE_ATTR_RO != 0,
                disk: self,
            });
        }

        files
    }

    fn read_sector(&self, sector_num: u16) -> Vec<u8> {
        let i: usize = (sector_num as usize) * SECTOR_SIZE;
        self.data[i..i + SECTOR_SIZE].to_vec()
    }
}


/// SF-7000 file
#[derive(Clone, Debug)]
pub struct File<'a> {
    /// File name
    pub name: String,

    /// Index of first cluster
    first_cluster: u8,

    /// File type
    pub file_type: FileType,

    /// Is the file read-only?
    pub readonly: bool,

    /// Reference to disk structure this file is on
    disk: &'a Disk,

}

const FAT_LAST_CLUSTER_MASK: u8		= 0xf0;
const FAT_LAST_CLUSTER_PREFIX: u8	= 0xc0;
const FAT_LAST_CLUSTER_NUM_SECTORS_MASK: u8 = 0x0f;
const FAT_RESERVED: u8			= 0xfe;
const FAT_UNUSED: u8			= 0xff;

impl File<'_> {
    fn read_cluster(&self, cluster_num: u8) -> Vec<u8> {
        let mut dest = Vec::with_capacity(CLUSTER_SIZE);

        for s in 0..SECTORS_PER_CLUSTER {
            dest.append(&mut self.disk.read_sector((((cluster_num as usize) * SECTORS_PER_CLUSTER) + s) as u16));
        }

        dest
    }

    fn fat_entry(&self, cluster_num: u8) -> u8 {
        self.disk.data[DISK_FAT_START + (cluster_num as usize)]
    }

    /// Read contents
    pub fn read(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut visited_clusters = HashSet::new();

        let mut cluster_num = self.first_cluster;
        while cluster_num < 160 {
            if visited_clusters.contains(&cluster_num) {
                eprintln!("FAT loop detected when reading file \"{}\".", self.name);
                return bytes;
            }
            visited_clusters.insert(cluster_num);

            let fat_entry = self.fat_entry(cluster_num);
            if fat_entry & FAT_LAST_CLUSTER_MASK == FAT_LAST_CLUSTER_PREFIX {
                let num_sectors = fat_entry & FAT_LAST_CLUSTER_NUM_SECTORS_MASK;
                let sector_num_start = (cluster_num as u16) * (SECTORS_PER_CLUSTER as u16);
                for i in 0..num_sectors {
                    bytes.append(&mut self.disk.read_sector(sector_num_start + i as u16));
                }
            } else {
                bytes.append(&mut self.read_cluster(cluster_num));
            }

            cluster_num = fat_entry;
        }

        bytes
    }

}
