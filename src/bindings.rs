#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[repr(C)]
pub struct in6_ifreq {
    pub ifr6_addr: nix::libc::in6_addr,
    pub ifr6_prefixlen: u32,
    pub ifr6_ifindex: i32,
}
