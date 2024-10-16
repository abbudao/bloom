// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use hyper::header::{parsing, Formatter, Header, Raw};
use hyper::Result;
use std::fmt;

#[derive(Clone)]
pub struct HeaderResponseBloomResponseTTL(pub usize);

impl Header for HeaderResponseBloomResponseTTL {
    fn header_name() -> &'static str {
        "Bloom-Response-TTL"
    }

    fn parse_header(raw: &Raw) -> Result<Self> {
        parsing::from_one_raw_str(raw).map(HeaderResponseBloomResponseTTL)
    }

    fn fmt_header(&self, f: &mut Formatter) -> fmt::Result {
        f.fmt_line(self)
    }
}

impl fmt::Display for HeaderResponseBloomResponseTTL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
