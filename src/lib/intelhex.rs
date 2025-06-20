use std::fs::read_to_string;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

use bytes::{Bytes, BytesMut, BufMut};

use hex;
use hex::ToHex;

#[derive(Debug)]
pub enum IntelHexFileError {
    InvalidRecordStart,
    InvalidRecordType,
    InvalidRecordLength
}

impl Error for IntelHexFileError {}

impl Display for IntelHexFileError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

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
    fn parse(s: &str) -> Result<Self, IntelHexFileError> {
        match s {
            "00" => Ok(Self::Data),
            "01" => Ok(Self::EndOfFile),
            "02" => Ok(Self::ExtendedSegmentAddress),
            "04" => Ok(Self::ExtendedLinearAddress),
            "05" => Ok(Self::StartLinearAddress),
            _   => Err(IntelHexFileError::InvalidRecordType)
        }
    }

    fn to_u8(&self) -> u8 {
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
    len: u8,
    addr: u16,
    rtype: RecordType,
    data: Bytes,
    checksum: u8
}

#[allow(unused)]
impl Record {
    pub fn parse(line: &str) -> Result<Option<Self>, Box<dyn Error>> {
        let start = match line.find(|b| b == RECORD_START) {
            Some(s) => s,
            None => return Ok(None)
        };

        let record_str = &line[start + 1..];

        let len: u8 = match hex::decode(&record_str[0..2]) {
            Ok(b) => b,
            Err(e) => return Err(Box::new(e))
        }.iter().sum();

        let data_end = 8 + (len as usize * 2);

        if !(data_end + 2 <= record_str.len()) {
            return Err(Box::new(IntelHexFileError::InvalidRecordLength))
        }

        Ok(Some(Record {
            len: len,

            addr: match hex::decode(&record_str[2..6]) {
                Ok(b) => (b[0] as u16 * 256) + b[1] as u16,
                Err(e) => return Err(Box::new(e))
            },

            rtype: match RecordType::parse(&record_str[6..8]) {
                Ok(r) => r,
                Err(e) => return Err(Box::new(e))
            },

            data: Bytes::from(match hex::decode(&record_str[8..data_end]) {
                Ok(b) => b,
                Err(e) => return Err(Box::new(e))
            }),

            checksum: match hex::decode(&record_str[data_end..data_end + 2]) {
                Ok(b) => b,
                Err(e) => return Err(Box::new(e))
            }.iter().sum()
        }))
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

}

#[allow(unused)]
pub struct IntelHexFile {
    pub path: String,
    pub size: usize,
    pub records: Vec<Record>
}

#[allow(unused)]
impl IntelHexFile {
    fn parse_records(raw_data: &str) -> Result<Vec<Record>, Box<dyn Error>> {
        let mut records = Vec::<Record>::new();

        for line in raw_data.lines() {
            let record_opt = match Record::parse(line) {
                Ok(o) => o,
                Err(e) => return Err(e)
            };

            match record_opt {
                Some(r) => records.push(r),
                None => continue
            }
        }

        return Ok(records);
    }

    pub fn load(path: &str) -> Result<IntelHexFile, Box<dyn Error>> {
        let raw_data: String = match read_to_string(path) {
            Ok(s) => s,
            Err(e) => return Err(Box::new(e))
        };

        Ok(IntelHexFile {
            path: path.to_string(),
            size: raw_data.bytes().len(),
            records: match Self::parse_records(&raw_data) {
                Ok(r) => r,
                Err(e) => return Err(e)
            }
        })
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
}
