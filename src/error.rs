use csv::Error;
use starknet::providers::ProviderError;
use std::io;
use url::ParseError;

#[derive(Debug)]
pub enum SerpicoError {
    UrlParsingErr(ParseError),
    ClientErr(ProviderError),
    WriterErr(Error),
    IoErr(io::Error),
}
