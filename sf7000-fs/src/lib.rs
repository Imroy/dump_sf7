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

use std::fmt;
use std::collections::HashMap;
use std::collections::HashSet;

use sc3000-charset::convert_utf8;

// Size of each disk (each side is read seperately)
pub const TRACKS_PER_DISK: usize = 40;
pub const SECTORS_PER_TRACK: usize = 16;
pub const SECTOR_SIZE: usize = 256;

// Derived sizes
pub const TRACK_SIZE: usize = SECTOR_SIZE * SECTORS_PER_TRACK;
pub const DISK_SIZE: usize = TRACK_SIZE * TRACKS_PER_DISK;

// 'Clusters' is used by the file system
pub const SECTORS_PER_CLUSTER: usize = 4;
pub const CLUSTER_SIZE: usize = SECTOR_SIZE * SECTORS_PER_CLUSTER;

// Disk structure constants
pub const DISK_ID_START: usize		= 0;
pub const DISK_ID_END: usize		= 4;

pub const DISK_NAME_START: usize	= 4;
pub const DISK_NAME_END: usize		= 32;

pub const DISK_IPL_START: usize		= 32;
pub const DISK_IPL_END: usize		= SECTOR_SIZE;

pub const DISK_RESERVED_0_START: usize	= SECTOR_SIZE;
pub const DISK_RESERVED_0_END: usize	= TRACK_SIZE;

pub const DISK_SYSTEM_PROGRAMS_START: usize = TRACK_SIZE;
pub const DISK_SYSTEM_PROGRAMS_END: usize = 20 * TRACK_SIZE;

pub const DISK_DIRECTORY_START: usize	= 20 * TRACK_SIZE;
pub const DISK_DIRECTORY_END: usize	= (20 * TRACK_SIZE) + (12 * SECTOR_SIZE);

pub const DISK_FAT_START: usize		= (20 * TRACK_SIZE) + (12 * SECTOR_SIZE);
pub const DISK_FAT_END: usize		= 21 * TRACK_SIZE;

pub const DISK_USER_START: usize	= 21 * TRACK_SIZE;
pub const DISK_USER_END: usize		= DISK_SIZE;

/// File types as stored on disk
#[derive(PartialEq)]
pub enum FileType {
    NonAscii = 0,
    Ascii = 1,
    Hexadecimal = 2,
}

impl TryFrom<u8> for FileType {
    type Error = ();
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            x if x == FileType::NonAscii as u8 => Ok(FileType::NonAscii),
            x if x == FileType::Ascii as u8 => Ok(FileType::Ascii),
            x if x == FileType::Hexadecimal as u8 => Ok(FileType::Hexadecimal),
            _ => Err(()),
        }
    }
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


struct DirEntry<'a> {
    name: &'a [u8],
    first_cluster: u8,
    file_type: FileType,
    readonly: bool,
}

const FILE_ATTR_TYPE_MASK: u8	= 0x0f;
const FILE_ATTR_RO: u8		= 0x80;

impl DirEntry<'_> {
    fn from_raw(data: &[u8]) -> DirEntry {
        DirEntry {
            name: &data[0..12],
            first_cluster: data[12],
            file_type: (data[13] & FILE_ATTR_TYPE_MASK).try_into().unwrap(),
            readonly: data[13] & FILE_ATTR_RO != 0,
        }
    }
}


pub const DIR_ENTRY_SIZE: usize = 16;
pub const MAX_DIR_ENTRIES: usize = (12 * SECTOR_SIZE) / DIR_ENTRY_SIZE;

/// SF-7000 disk image
pub struct Disk {
    data: Vec<u8>,
}

/// SF-7000 file
pub struct File<'a> {
    raw_name: Vec<u8>,
    pub name: String,
    pub first_cluster: u8,
    pub file_type: FileType,
    pub readonly: bool,
    disk: &'a Disk,
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
        let i = DISK_ID_START as usize;
        return (self.data[i] == b'S')
            && (self.data[i + 1] == b'Y')
            && (self.data[i + 2] == b'S')
            && (self.data[i + 3] == b':');
    }

    /// Disk name
    pub fn name(&self, charmap: &HashMap<u8, char>) -> String {
        return convert_utf8(&self.data[DISK_NAME_START..DISK_NAME_END], charmap);
    }

    /// Initial Program Loader
    pub fn ipl(&self) -> &[u8] {
        return &self.data[DISK_IPL_START..DISK_IPL_END];
    }

    /// List the files on disk
    pub fn list_directory(&self, charmap: &HashMap<u8, char>) -> Vec<File> {
        let mut files: Vec<File> = Vec::new();

        for i in 0..MAX_DIR_ENTRIES {
            let entry_start = DISK_DIRECTORY_START + (i * DIR_ENTRY_SIZE);
            let entry = DirEntry::from_raw(&self.data[entry_start..entry_start + DIR_ENTRY_SIZE]);
            if entry.name[0] == b'\0' {
                continue;
            }

            let mut utf8_filename = convert_utf8(entry.name, charmap);
            let (mut name_part, mut ext_part) = utf8_filename.split_at(8);
            while name_part.ends_with(" ") {
                name_part = name_part.strip_suffix(' ').unwrap();
            }
            while ext_part.ends_with(" ") {
                ext_part = ext_part.strip_suffix(' ').unwrap();
            }
            utf8_filename = name_part.to_owned() + ext_part;
            utf8_filename = utf8_filename.replace("/", "--");

            files.push(File {
                raw_name: entry.name.to_vec(),
                name: utf8_filename,
                first_cluster: entry.first_cluster,
                file_type: entry.file_type,
                readonly: entry.readonly,
                disk: self,
            });
        }

        return files;
    }

    fn read_sector(&self, sector_num: u16) -> Vec<u8> {
        let i: usize = (sector_num as usize) * SECTOR_SIZE;
        self.data[i..i + SECTOR_SIZE].to_vec()
    }
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
