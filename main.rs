// ```
// cargo run --release
// cargo run --release --features=rtt
// ```
#![no_main]
#![no_std]

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

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("RTT initialization done.");
    rprint!("Here goes a mandatory message: ");
    rprintln!("Hello, World!");

    let device = unsafe { pac::Peripherals::steal() };

    // Define pin bits for P003 and P004
    let p003_bit = 1 << 3; // Bit 3 for P003
    let p004_bit = 1 << 4; // Bit 4 for P004

    // Configure P003 and P004 for LED matrix control
    // Initially set both as inputs (high-impedance)
    device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(0) }); // Clear all P0 direction bits
    device.PORT0.podr().write(|w| unsafe { w.podr().bits(0) }); // Clear all P0 output bits

    loop {
        // Light LED 1 (P003 High, P004 Low)
        rprintln!("Lighting LED 1 (P003 High, P004 Low)");
        device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(p003_bit | p004_bit) }); // Set both as outputs
        device.PORT0.podr().write(|w| unsafe { w.podr().bits(p003_bit) }); // P003 High, P004 Low
        nop_delay(500000);

        // Light LED 2 (P004 High, P003 Low)
        rprintln!("Lighting LED 2 (P004 High, P003 Low)");
        device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(p003_bit | p004_bit) }); // Set both as outputs
        device.PORT0.podr().write(|w| unsafe { w.podr().bits(p004_bit) }); // P004 High, P003 Low
        nop_delay(500000);

        // Turn off LEDs (both as inputs)
        rprintln!("Turning off LEDs (both inputs)");
        device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(0) }); // Set both as inputs
        nop_delay(500000);
    }
}


fn nop_delay(i:u32)
{
        for _ in 0..i {
            nop();
        }
}
