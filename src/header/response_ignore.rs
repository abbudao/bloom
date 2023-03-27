// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)
extern crate headers;
use headers::{Header, HeaderName, HeaderValue};
use std::fmt;

#[derive(Clone)]
pub struct HeaderResponseBloomResponseIgnore();

impl Header for HeaderResponseBloomResponseIgnore {
    fn name() -> &'static HeaderName {
        "Bloom-Response-Ignore"
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;

        if value == "1" {
            Ok(HeaderResponseBloomResponseIgnore())
        } else {
            Err(headers::Error::invalid())
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

impl fmt::Display for HeaderResponseBloomResponseIgnore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&1, f)
    }
}
