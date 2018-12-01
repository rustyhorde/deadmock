use std::fmt;

#[derive(Debug)]
crate enum Error {
    AddrParse(std::net::AddrParseError),
    Clap(clap::Error),
    Failure(failure::Error),
    Libdeadmock(libdeadmock::error::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AddrParse(e) => write!(f, "{}", e),
            Error::Clap(e) => write!(f, "{}", e),
            Error::Failure(e) => write!(f, "{}", e),
            Error::Libdeadmock(e) => write!(f, "{}", e),
        }
    }
}

macro_rules! from_err {
    ($x:ty, $y:ident) => {
        impl From<$x> for Error {
            fn from(e: $x) -> Self {
                Error::$y(e)
            }
        }
    };
}

from_err!(std::net::AddrParseError, AddrParse);
from_err!(clap::Error, Clap);
from_err!(failure::Error, Failure);
from_err!(libdeadmock::error::Error, Libdeadmock);
