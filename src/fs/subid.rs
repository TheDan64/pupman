pub const ETC_SUBGID: &str = "/etc/subgid";
pub const ETC_SUBUID: &str = "/etc/subuid";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubID {
    UID,
    GID,
}
