pub fn get_api_url(domain: &str) -> String {
    let host_with_port = if domain.starts_with('[') {
        // IPv6 literal
        match domain.find(']') {
            Some(closing) => {
                let after = &domain[closing + 1..];
                if after.starts_with(':') {
                    domain.to_string()
                } else {
                    format!("{domain}:7000")
                }
            }
            None => {
                // malformed IPv6, just append
                format!("{domain}:7000")
            }
        }
    } else if domain.contains(':') {
        domain.to_string()
    } else {
        format!("{domain}:7000")
    };

    format!("http://{host_with_port}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_port() {
        let url = get_api_url("example.com");
        assert_eq!(url, "http://example.com:7000");
    }

    #[test]
    fn test_with_port() {
        let url = get_api_url("example.com:8080");
        assert_eq!(url, "http://example.com:8080");
    }

    #[test]
    fn test_ipv6_no_port() {
        let url = get_api_url("[::1]");
        assert_eq!(url, "http://[::1]:7000");
    }

    #[test]
    fn test_ipv6_with_port() {
        let url = get_api_url("[::1]:4321");
        assert_eq!(url, "http://[::1]:4321");
    }

    #[test]
    fn test_malformed_ipv6() {
        // no closing ']', treated as no port
        let url = get_api_url("[::1");
        assert_eq!(url, "http://[::1:7000");
    }
}
