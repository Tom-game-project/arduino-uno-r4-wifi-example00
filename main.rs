// ```
// cargo run --release
// cargo run --release --features=rtt
// sudo chmod a+rw /dev/hidraw2
// ```
#![no_main]
#![no_std]

use cortex_m::Peripherals;
#[cfg(feature = "rtt")]
use panic_rtt_target as _;
use ra4m1 as pac;
mod setting;

#[allow(unused)]
use cortex_m::asm::nop;

#[cfg(feature="rtt")]
use rtt_target::{rprint, rprintln, rtt_init_print};

#[cfg(not(feature="rtt"))]
macro_rules! rtt_init_print { () => { } }
#[cfg(not(feature="rtt"))]
macro_rules! rprint { ($($arg:tt)*) => { let _ = ($($arg)*); } }
#[cfg(not(feature="rtt"))]
macro_rules! rprintln { ($($arg:tt)*) => { let _ = ($($arg)*); } }
#[cfg(not(feature = "rtt"))]
#[inline(never)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::asm::udf();
}

#[inline(never)]
pub fn exit() -> ! {
    cortex_m::asm::bkpt();
    loop { }
}

// LED MATRIX CODE

// Define pin bits for LED matrix control
// P0xx pins
const P003_BIT: u16 = 1 << 3;
const P004_BIT: u16 = 1 << 4;
const P011_BIT: u16 = 1 << 11;
const P012_BIT: u16 = 1 << 12;
const P013_BIT: u16 = 1 << 13;
const P015_BIT: u16 = 1 << 15;

// P2xx pins
const P204_BIT: u16 = 1 << 4;
const P205_BIT: u16 = 1 << 5;
const P206_BIT: u16 = 1 << 6;
const P212_BIT: u16 = 1 << 12;
const P213_BIT: u16 = 1 << 13;

const ss_list: [(u16, u16); 96]= 
[
    (P205_BIT, P012_BIT),
    (P012_BIT, P205_BIT),
    (P205_BIT, P013_BIT),
    (P013_BIT, P205_BIT),
    (P012_BIT, P013_BIT),
    (P013_BIT, P012_BIT),
    (P205_BIT, P003_BIT),
    (P003_BIT, P205_BIT),
    (P012_BIT, P003_BIT),
    (P003_BIT, P012_BIT),
    (P013_BIT, P003_BIT),
    (P003_BIT, P013_BIT),
    (P012_BIT, P004_BIT),
    (P004_BIT, P012_BIT),
    (P013_BIT, P004_BIT),
    (P004_BIT, P013_BIT),
    (P003_BIT, P004_BIT),
    (P004_BIT, P003_BIT),
    (P205_BIT, P011_BIT),
    (P011_BIT, P205_BIT),
    (P012_BIT, P011_BIT),
    (P011_BIT, P012_BIT),
    (P013_BIT, P011_BIT),
    (P011_BIT, P013_BIT),
    (P205_BIT, P004_BIT),
    (P004_BIT, P205_BIT),
    (P012_BIT, P011_BIT),
    (P011_BIT, P012_BIT),
    (P013_BIT, P011_BIT),
    (P011_BIT, P013_BIT),
    (P003_BIT, P011_BIT),
    (P011_BIT, P003_BIT),
    (P004_BIT, P011_BIT),
    (P011_BIT, P004_BIT),
    (P205_BIT, P015_BIT),
    (P015_BIT, P205_BIT),
    (P004_BIT, P205_BIT),
    (P205_BIT, P004_BIT),
    (P011_BIT, P012_BIT),
    (P012_BIT, P011_BIT),
    (P003_BIT, P015_BIT),
    (P015_BIT, P003_BIT),
    (P004_BIT, P015_BIT),
    (P015_BIT, P004_BIT),
    (P011_BIT, P015_BIT),
    (P015_BIT, P011_BIT),
    (P205_BIT, P204_BIT),
    (P204_BIT, P205_BIT),
    (P004_BIT, P204_BIT),
    (P204_BIT, P004_BIT),
    (P011_BIT, P204_BIT),
    (P204_BIT, P011_BIT),
    (P015_BIT, P204_BIT),
    (P204_BIT, P015_BIT),
    (P205_BIT, P206_BIT),
    (P206_BIT, P205_BIT),
    (P012_BIT, P206_BIT),
    (P206_BIT, P012_BIT),
    (P013_BIT, P206_BIT),
    (P206_BIT, P013_BIT),
    (P204_BIT, P004_BIT),
    (P004_BIT, P204_BIT),
    (P204_BIT, P011_BIT),
    (P011_BIT, P204_BIT),
    (P204_BIT, P015_BIT),
    (P015_BIT, P204_BIT),
    (P206_BIT, P205_BIT),
    (P205_BIT, P206_BIT),
    (P206_BIT, P012_BIT),
    (P012_BIT, P206_BIT),
    (P206_BIT, P013_BIT),
    (P013_BIT, P206_BIT),
    (P205_BIT, P212_BIT),
    (P212_BIT, P205_BIT),
    (P012_BIT, P212_BIT),
    (P212_BIT, P012_BIT),
    (P013_BIT, P212_BIT),
    (P212_BIT, P013_BIT),
    (P003_BIT, P212_BIT),
    (P212_BIT, P003_BIT),
    (P004_BIT, P212_BIT),
    (P212_BIT, P004_BIT),
    (P011_BIT, P212_BIT),
    (P212_BIT, P011_BIT),
    (P205_BIT, P213_BIT),
    (P213_BIT, P205_BIT),
    (P012_BIT, P213_BIT),
    (P213_BIT, P012_BIT),
    (P013_BIT, P213_BIT),
    (P213_BIT, P013_BIT),
    (P003_BIT, P213_BIT),
    (P213_BIT, P003_BIT),
    (P004_BIT, P213_BIT),
    (P213_BIT, P004_BIT),
    (P011_BIT, P213_BIT),
    (P213_BIT, P011_BIT),
];

fn show_led_matrix(device: &ra4m1::Peripherals, x:usize, y:usize)
{
    rprintln!("show_led_matrix {}", x + y * 12);
    device.PORT0.pdr().write(|w| unsafe {
        w.pdr().bits(ss_list[x + y * 12].0 | ss_list[x + y * 12].1)
    }); // Set both as outputs
    device.PORT0.podr().write(|w| unsafe { w.podr().bits(ss_list[x + y * 12].1) }); 
    return ;
}

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("RTT initialization done.");
    rprint!("Here goes a mandatory message: ");
    rprintln!("Hello, World!");

    let device = unsafe { pac::Peripherals::steal() };

    // Define pin bits for P003 and P004
    //P003_BIT
    //let p003_bit = 1 << 3; // Bit 3 for P003
    //let p004_bit = 1 << 4; // Bit 4 for P004

    // Configure P003 and P004 for LED matrix control
    // Initially set both as inputs (high-impedance)
    device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(0) }); // Clear all P0 direction bits
    device.PORT0.podr().write(|w| unsafe { w.podr().bits(0) }); // Clear all P0 output bits

    loop {
        // Light LED 1 (P003 High, P004 Low)
        //rprintln!("Lighting LED 1 (P003 High, P004 Low)");
        //device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(p003_bit | p004_bit) }); // Set both as outputs
        //device.PORT0.podr().write(|w| unsafe { w.podr().bits(p003_bit) }); // P003 High, P004 Low
        //
        //nop_delay(100000);
        //
        //// Light LED 2 (P004 High, P003 Low)
        //rprintln!("Lighting LED 2 (P004 High, P003 Low)");
        //device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(p003_bit | p004_bit) }); // Set both as outputs
        //device.PORT0.podr().write(|w| unsafe { w.podr().bits(p004_bit) }); // P004 High, P003 Low
        //nop_delay(100000);
        //
        //// Turn off LEDs (both as inputs)
        //rprintln!("Turning off LEDs (both inputs)");
        //device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(0) }); // Set both as inputs
        //nop_delay(100000);
        //

        show_led_matrix(&device, 54, 0);
        //show_led_matrix(&device, 6, 6);
        //show_led_matrix(&device, 1, 1);
        //show_led_matrix(&device, 2, 2);
        nop_delay(100000);
        rprintln!("Turning off LEDs (both inputs)");
        device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(0) }); // Set both as inputs
        nop_delay(100000);
    }
}


fn nop_delay(i:u32)
{
        for _ in 0..i {
            nop();
        }
}
