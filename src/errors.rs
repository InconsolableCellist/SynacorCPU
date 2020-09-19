use std::fmt;
#[derive(Debug)]

pub enum Error {
    MemoryInvalid,
    UnknownOpcode,
    EmptyStack,
    FailedToReadLine,
}

impl fmt::Display for Error {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match self {
            MemoryInvalid => write!(f, "Invalid memory access"),
            UnknownOpcode => write!(f, "Unknown opcode"),
            EmptyStack => write!(f, "Attempted to pop off of an empty stack"),
            FailedToReadLine => write!(f, "Failed to read line from STDIN"),
        }
    }
}

impl std::error::Error for Error {}