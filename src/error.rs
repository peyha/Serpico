use starknet::providers::ProviderError;
use url::ParseError;

#[derive(Debug)]
pub enum SerpicoError {
    UrlParsingErr(ParseError),
    ClientErr(ProviderError),
}
