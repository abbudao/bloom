// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use headers::{Header, HeaderName, HeaderValue};
use std::fmt;

#[derive(Clone)]
pub struct HeaderResponseBloomResponseBuckets(pub Vec<String>);

impl Header for HeaderResponseBloomResponseBuckets {
    fn name() -> &'static HeaderName {
        &HeaderName::from_static("Bloom-Response-Buckets")
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let mut result = Vec::new();
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        result.extend(
            value
                .split(',')
                .filter_map(|x| match x.trim() {
                    "" => None,
                    y => Some(y),
                })
                .filter_map(|x| x.parse().ok()),
        );

        Ok(result)
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        values.extend(self.0.join(","));
    }
}

impl fmt::Display for HeaderResponseBloomResponseBuckets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f(self.0.join(","))?;
        Ok(())
    }
}
