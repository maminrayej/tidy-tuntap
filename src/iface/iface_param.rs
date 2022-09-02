use crate::iface;

// TODO: use tidy_builder when the `default` feature is landed.
pub(crate) struct InterfaceParams<'a> {
    pub(crate) name: &'a str,
    pub(crate) mode: iface::Mode,
    pub(crate) fd_count: usize,
    pub(crate) non_blocking: bool,
    pub(crate) no_packet_info: bool,
}
