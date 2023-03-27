// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use headers::{Header, HeaderName, HeaderValue};
use std::fmt;

#[derive(Clone)]
pub struct HeaderResponseBloomResponseTTL(pub usize);

impl Header for HeaderResponseBloomResponseTTL {
    fn name() -> &'static HeaderName {
        &HeaderName::from_static("Bloom-Response-TTL")
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;

        usize::from_str(value).map_err(|| headers::Error::invalid())
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let value = HeaderValue::from_static(self.0);
        values.extend(std::iter::once(value));
    }
}

impl fmt::Display for HeaderResponseBloomResponseTTL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
