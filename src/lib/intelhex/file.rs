use std::fs::{read_to_string, write};
use std::fmt::Debug;

use bytes::{Bytes, BytesMut, BufMut};

use hex;
use hex::ToHex;

use crate::util::twos_comp;
use crate::error::{IntelHexError, IHexError};

const RECORD_START: char = ':';

#[derive(Debug)]
pub enum RecordType {
    Data,
    EndOfFile,
    ExtendedSegmentAddress,
    ExtendedLinearAddress,
    StartLinearAddress
}

#[allow(unused)]
impl RecordType {
    fn parse(s: &str) -> Result<Self, IntelHexError> {
        match s {
            "00" => Ok(Self::Data),
            "01" => Ok(Self::EndOfFile),
            "02" => Ok(Self::ExtendedSegmentAddress),
            "04" => Ok(Self::ExtendedLinearAddress),
            "05" => Ok(Self::StartLinearAddress),
            _   => Err(IHexError::RecordInvalidType.new(
                &format!("Invalid record type: {}", s)
            ))
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Data => 0,
            Self::EndOfFile => 1,
            Self::ExtendedSegmentAddress => 2,
            Self::ExtendedLinearAddress => 4,
            Self::StartLinearAddress => 5
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Record {
    pub len: u8,
    pub addr: u16,
    pub rtype: RecordType,
    pub data: Bytes,
    pub checksum: u8
}

#[allow(unused)]
impl Record {
    pub fn parse(line: &str) -> Result<Option<Self>, IntelHexError> {
        let start = match line.find(|b| b == RECORD_START) {
            Some(index) => index,
            None => return Ok(None)
        };

        let record_str = &line[start + 1..];
        
        let len: u8 = match hex::decode(&record_str[0..2]) {
            Ok(byte) => byte,
            Err(err) => {
                return Err(
                    IHexError::RecordBadEndcoding.new(
                        "Error while decoding record length"
                    ).set_source(Box::new(err)))
            }
        }.iter().sum();

        let data_end = 8 + (len as usize * 2);
        let record_end = 2 + data_end;

        if !(record_end <= record_str.len()) {
            return Err(IHexError::RecordInvalidLength.new(&format!(
                "Record length: {}, expected: {}", record_str.len(), record_end,
            )))
        }

        let record = Record {
            len: len,

            addr: match hex::decode(&record_str[2..6]) {
                Ok(byts) => (byts[0] as u16 * 256) + byts[1] as u16,
                Err(err) => return Err(
                    IHexError::RecordBadEndcoding.new(
                        "Error while decoding address"
                    ).set_source(Box::new(err))
                )
            },

            rtype: match RecordType::parse(&record_str[6..8]) {
                Ok(rtype) => rtype,
                Err(err) => return Err(err)
            },

            data: Bytes::from(match hex::decode(&record_str[8..data_end]) {
                Ok(byts) => byts,
                Err(err) => return Err(
                    IHexError::RecordBadEndcoding.new(
                        "Error while decoding address"
                    ).set_source(Box::new(err))
                )
            }),

            checksum: match hex::decode(&record_str[data_end..record_end]) {
                Ok(byts) => byts,
                Err(err) => return Err(
                    IHexError::RecordBadEndcoding.new(
                        "Error while decoding checksum"
                    ).set_source(Box::new(err))
                )
            }.iter().sum()
        };

        let checksum = record.calculate_checksum();
        match checksum == record.checksum {
            true => Ok(Some(record)),
            false => Err(IHexError::RecordBadChecksum.new(&format!(
                "Bad checksum: 0x{:X}, calculated: 0x{:X} ({})",
                record.checksum, checksum, checksum
            )))
        }
    }
    
    pub fn binary_size(&self) -> usize {
       self.data.len() + 7
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut b = BytesMut::new();
        
        b.put_u8(self.len);
        b.put(&self.addr.to_be_bytes()[..]);
        b.put_u8(self.rtype.to_u8());
        b.put(self.data.clone());
        b.put_u8(self.checksum);
        
        return b.into();
    }

    pub fn to_hex_str(&self) -> String {
        let mut hex_str = self.to_bytes()
            .encode_hex::<String>()
            .to_ascii_uppercase();

        hex_str.insert(0, RECORD_START);

        return hex_str;
    }
    
    pub fn calculate_checksum(&self) -> u8 {
        let byts = self.to_bytes();
        twos_comp(
            byts[0..byts.len() - 1]
                .iter()
                .map(|b| *b as u64)
                .sum::<u64>() % 256
        )
    }

}

#[allow(unused)]
pub struct IntelHexFile {
    pub path: Option<String>,
    pub size: usize,
    pub records: Vec<Record>
}

#[allow(unused)]
impl IntelHexFile {
    fn parse_records(raw_data: &str) -> Result<Vec<Record>, IntelHexError> {
        let mut records = Vec::<Record>::new();

        for (i, line) in raw_data.lines().enumerate() {
            let record_opt = match Record::parse(line) {
                Ok(opt) => opt,
                Err(err) => return Err(IHexError::FileBadRecord.new(
                    &format!("Error while parsing record on line {}", i + 1)
                ).set_source(Box::new(err)))
            };

            match record_opt {
                Some(record) => records.push(record),
                None => continue
            }
        }

        return Ok(records);
    }

    pub fn load(raw_data: &str) -> Result<Self, IntelHexError> {
        Ok(Self {
            path: None,
            size: raw_data.bytes().len(),
            records: match Self::parse_records(&raw_data) {
                Ok(records) => records,
                Err(err) => return Err(IHexError::FileErrorLoad.new(
                    "Error loading data"
                ).set_source(Box::new(err)))
            }
        })
    }

    pub fn load_file(path: &str) -> Result<Self, IntelHexError> {
        let raw_data: String = match read_to_string(path) {
            Ok(string) => string,
            Err(err) => return Err(IHexError::FileErrorOpen.new(
                &format!("Error opening file: {}", path)
            ).set_source(Box::new(err)))
        };

        let mut intel_hex_file = match Self::load(&raw_data) {
            Ok(file) => file,
            Err(err_box) => return Err(err_box)
        };

        intel_hex_file.path = Some(path.to_string());

        return Ok(intel_hex_file)
    }

    pub fn binary_size(&self) -> usize {
        self.records.iter().map(|r| r.binary_size()).sum()
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut b = BytesMut::new();

        for record in &self.records {
            b.put(record.to_bytes())
        }

        return b.into();
    }

    pub fn to_hex_str(&self) -> String {
        let mut hex_str = String::new();
        let last_i = self.records.len() - 1;

        for (i, record) in self.records.iter().enumerate() {
            hex_str += &record.to_hex_str().clone();
            if i != last_i {
                hex_str += "\n";    
            }
        }

        return hex_str;
    }
    
    pub fn get_path(&self) -> String {
        self.path.clone().unwrap_or("(none)".to_string())
    }

    pub fn save_file(&mut self, path: &str) -> Result<(), IntelHexError> {
        match write(path, self.to_hex_str()) {
            Ok(_) => Ok(()),
            Err(err) => Err(IHexError::FileErrorWrite.new(
                &format!("Error while writing file: {}", path)
            ).set_source(Box::new(err)))
        }
    }
}
