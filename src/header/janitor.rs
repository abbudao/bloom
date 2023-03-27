// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use headers::Header;
use hyper::http::HeaderMap;

use super::response_buckets::HeaderResponseBloomResponseBuckets;
use super::response_ignore::HeaderResponseBloomResponseIgnore;
use super::response_ttl::HeaderResponseBloomResponseTTL;

pub struct HeaderJanitor;

impl HeaderJanitor {
    pub fn clean(headers: &mut HeaderMap) {
        // Map headers to clean-up
        let mut headers_remove: Vec<String> = Vec::new();

        for (name, value) in headers.iter() {
            // Do not forward contextual and internal headers (ie. 'Bloom-Response-*' headers)
            if Self::is_contextual(&name) || Self::is_internal(&name) {
                headers_remove.push(name.to_string());
            }
        }

        // Proceed headers clean-up
        for header_remove in &headers_remove {
            headers.remove_raw(header_remove.as_ref());
        }
    }

    pub fn is_contextual(header_name: &str) -> bool {
        header_name == headers::Connection::name()
            || header_name == headers::Date::name()
            || header_name == headers::Upgrade::name()
            || header_name == headers::Date::name()
    }

    pub fn is_interal(header_name: &str) -> bool {
        header_name == HeaderResponseBloomResponseBuckets::name()
            || header_name == HeaderResponseBloomResponseIgnore::name()
            || header_name == HeaderResponseBloomResponseTTL::name()
    }
}
