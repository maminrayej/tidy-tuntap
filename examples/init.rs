use std::io::stdin;

use tidy_tuntap::iface;

fn main() {
    let iface = iface::Interface::new("tun10", iface::Mode::Tun, true).unwrap();
    iface.bring_up().unwrap();

    let _ = stdin()
        .read_line(&mut String::new())
        .expect("Failed to read the user input");

    iface.bring_down().unwrap();

    let _ = stdin()
        .read_line(&mut String::new())
        .expect("Failed to read the user input");

    println!("Name of the device is: {}", iface.name());
}
