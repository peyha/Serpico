use csv::Error;
use starknet::providers::ProviderError;
use std::io;
use std::num::ParseIntError;
use url::ParseError;

#[derive(Debug)]
pub enum SerpicoError {
    UrlParsingErr(ParseError),
    IntParsingErr(ParseIntError),
    ClientErr(ProviderError),
    WriterErr(Error),
    IoErr(io::Error),
}
