use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Debug};

pub struct IntelHexError {
    msg: String,
    err_type: IHexError,
    source: Option<Box<dyn Error>>
}

impl IntelHexError {
    pub fn set_source(mut self, source: Box<dyn Error>) -> IntelHexError {
        self.source = Some(source);
        return self
    }
}

impl Display for IntelHexError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "IntelHexError")
    }
}

impl Debug for IntelHexError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match &self.source {
            Some(err_box) => write!(
                formatter,
                "IntelHexError({:?}): {}\n{:?}\n", 
                self.err_type, self.msg, err_box
            ),
            None => write!(
                formatter, "IntelHexError({:?}): {}",
                self.err_type, self.msg
            )
        }
    }
}

impl Error for IntelHexError {}

#[derive(Debug)]
pub enum IHexError {
    RecordInvalidStart,
    RecordInvalidType,
    RecordInvalidLength,
    RecordBadChecksum,
    RecordBadEndcoding,
    FileBadRecord,
    FileErrorLoad,
    FileErrorOpen,
    FileErrorWrite
}

impl IHexError {
    pub fn new(self, msg:&str) -> IntelHexError {
        IntelHexError {
            msg: msg.to_string(),
            err_type: self,
            source: None
        }
    }
}

impl Display for IHexError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

impl Error for IHexError {}
