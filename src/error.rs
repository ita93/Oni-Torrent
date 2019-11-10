use std::io::Error as StdIoError;
use std::fmt;
use serde_bencode::Error as BencodeError;
use std::net::AddrParseError as AddrParserError;
use bincode::Error as BincodeErrorKind;
use url::ParseError as EUrlParser;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(StdIoError),
    SerdeBencode(BencodeError),
    AddrParserError(AddrParserError),
    NotSupportProtocol(String),
    BincodeError(BincodeErrorKind),
    UrlError(EUrlParser),
    Unknown,
}

impl From<StdIoError> for Error{
    fn from(err: StdIoError) -> Error{
        Error::Io(err)
    }
}

impl From<BencodeError> for Error {
    fn from(err: BencodeError) -> Error {
        Error::SerdeBencode(err)
    }
}

impl From<AddrParserError> for Error {
    fn from(err: AddrParserError) -> Error {
        Error::AddrParserError(err)
    }
}

impl From<BincodeErrorKind> for Error {
    fn from(err: BincodeErrorKind) -> Error {
        Error::BincodeError(err)
    }

}

impl From<EUrlParser> for Error {
    fn from(err: EUrlParser) -> Error {
        Error::UrlError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result<> {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::SerdeBencode(ref err) => err.fmt(f),
            Error::NotSupportProtocol(ref s) => f.write_str(s),
            Error::AddrParserError(ref err) => err.fmt(f),
            Error::BincodeError(ref err) => err.fmt(f),
            Error::UrlError(ref err) => err.fmt(f),
            _ => f.write_str("An unknown Error just happend."),
        }
    }
}
