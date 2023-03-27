// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)
extern crate headers;
use headers::{Header, HeaderName, HeaderValue};
use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub struct HeaderRequestBloomRequestShard(pub u8);

impl Header for HeaderRequestBloomRequestShard {
    fn name() -> &'static HeaderName {
        &HeaderName::from_static("Bloom-Request-Shard")
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        let str_value = value.to_str().map_err(|_| headers::Error::invalid())?;

        match u8::from_str(str_value) {
            Ok(v) => Ok(HeaderRequestBloomRequestShard(v)),
            _ => Err(headers::Error::invalid()),
        }
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let value = HeaderValue::from_static(self.0);
        values.extend(std::iter::once(value));
    }
}

impl fmt::Display for HeaderRequestBloomRequestShard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
