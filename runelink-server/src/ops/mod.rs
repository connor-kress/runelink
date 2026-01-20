pub mod channels;
pub mod hosts;
pub mod memberships;
pub mod messages;
pub mod servers;
pub mod users;

use runelink_client::util::pad_domain;

pub fn is_remote_domain(domain: Option<&str>, local_domain: &str) -> bool {
    let Some(domain) = domain else {
        return false;
    };
    pad_domain(domain) != pad_domain(local_domain)
}
