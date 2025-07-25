// ```
// cargo run --release
// # With rtt output, it slows down and the LED flickers, so it is for testing purposes.
// cargo run --release --features=rtt
// # Temporarily allow writing to the connected microcomputer device.
// sudo chmod a+rw /dev/hidraw*
// ```
#![no_main]
#![no_std]

use cortex_m::Peripherals;
#[cfg(feature = "rtt")]
use panic_rtt_target as _;
use ra4m1 as pac;
mod setting;
mod sci;

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

enum Ports
{
    Port0xxPins(u16),
    Port2xxPins(u16),
}

// Define pin bits for LED matrix control
// P0xx pins
const P003_BIT:Ports = Ports::Port0xxPins(1 << 3);
const P004_BIT:Ports = Ports::Port0xxPins(1 << 4);
const P011_BIT:Ports = Ports::Port0xxPins(1 << 11);
const P012_BIT:Ports = Ports::Port0xxPins(1 << 12);
const P013_BIT:Ports = Ports::Port0xxPins(1 << 13);
const P015_BIT:Ports = Ports::Port0xxPins(1 << 15);

// P2xx pins
const P204_BIT:Ports = Ports::Port2xxPins(1 << 4);
const P205_BIT:Ports = Ports::Port2xxPins(1 << 5);
const P206_BIT:Ports = Ports::Port2xxPins(1 << 6);
const P212_BIT:Ports = Ports::Port2xxPins(1 << 12);
const P213_BIT:Ports = Ports::Port2xxPins(1 << 13);

const PIN_LIST: [(Ports, Ports); 96] =
[
    (P012_BIT, P205_BIT),
    (P205_BIT, P012_BIT),
    (P013_BIT, P205_BIT),
    (P205_BIT, P013_BIT),
    (P013_BIT, P012_BIT),
    (P012_BIT, P013_BIT),
    (P206_BIT, P205_BIT),
    (P205_BIT, P206_BIT),
    (P206_BIT, P012_BIT),
    (P012_BIT, P206_BIT),
    (P206_BIT, P013_BIT),
    (P013_BIT, P206_BIT),
    (P003_BIT, P205_BIT),
    (P205_BIT, P003_BIT),
    (P003_BIT, P012_BIT),
    (P012_BIT, P003_BIT),
    (P003_BIT, P013_BIT),
    (P013_BIT, P003_BIT),
    (P003_BIT, P206_BIT),
    (P206_BIT, P003_BIT),
    (P204_BIT, P205_BIT),
    (P205_BIT, P204_BIT),
    (P204_BIT, P012_BIT),
    (P012_BIT, P204_BIT),
    (P204_BIT, P013_BIT),
    (P013_BIT, P204_BIT),
    (P204_BIT, P206_BIT),
    (P206_BIT, P204_BIT),
    (P204_BIT, P003_BIT),
    (P003_BIT, P204_BIT),
    (P015_BIT, P205_BIT),
    (P205_BIT, P015_BIT),
    (P015_BIT, P012_BIT),
    (P012_BIT, P015_BIT),
    (P015_BIT, P013_BIT),
    (P013_BIT, P015_BIT),
    (P015_BIT, P206_BIT),
    (P206_BIT, P015_BIT),
    (P015_BIT, P003_BIT),
    (P003_BIT, P015_BIT),
    (P015_BIT, P204_BIT),
    (P204_BIT, P015_BIT),
    (P004_BIT, P205_BIT),
    (P205_BIT, P004_BIT),
    (P004_BIT, P012_BIT),
    (P012_BIT, P004_BIT),
    (P004_BIT, P013_BIT),
    (P013_BIT, P004_BIT),
    (P004_BIT, P206_BIT),
    (P206_BIT, P004_BIT),
    (P004_BIT, P003_BIT),
    (P003_BIT, P004_BIT),
    (P004_BIT, P204_BIT),
    (P204_BIT, P004_BIT),
    (P004_BIT, P015_BIT),
    (P015_BIT, P004_BIT),
    (P011_BIT, P205_BIT),
    (P205_BIT, P011_BIT),
    (P011_BIT, P012_BIT),
    (P012_BIT, P011_BIT),
    (P011_BIT, P013_BIT),
    (P013_BIT, P011_BIT),
    (P011_BIT, P206_BIT),
    (P206_BIT, P011_BIT),
    (P011_BIT, P003_BIT),
    (P003_BIT, P011_BIT),
    (P011_BIT, P204_BIT),
    (P204_BIT, P011_BIT),
    (P011_BIT, P015_BIT),
    (P015_BIT, P011_BIT),
    (P011_BIT, P004_BIT),
    (P004_BIT, P011_BIT),
    (P213_BIT, P205_BIT),
    (P205_BIT, P213_BIT),
    (P213_BIT, P012_BIT),
    (P012_BIT, P213_BIT),
    (P213_BIT, P013_BIT),
    (P013_BIT, P213_BIT),
    (P213_BIT, P206_BIT),
    (P206_BIT, P213_BIT),
    (P213_BIT, P003_BIT),
    (P003_BIT, P213_BIT),
    (P213_BIT, P204_BIT),
    (P204_BIT, P213_BIT),
    (P213_BIT, P015_BIT),
    (P015_BIT, P213_BIT),
    (P213_BIT, P004_BIT),
    (P004_BIT, P213_BIT),
    (P213_BIT, P011_BIT),
    (P011_BIT, P213_BIT),
    (P212_BIT, P205_BIT),
    (P205_BIT, P212_BIT),
    (P212_BIT, P012_BIT),
    (P012_BIT, P212_BIT),
    (P212_BIT, P013_BIT),
    (P013_BIT, P212_BIT),
];

fn turnon_led_matrix(device: &ra4m1::Peripherals, x:usize, y:usize)
{
    device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(0) }); // Clear all P0 direction bits
    device.PORT2.pdr().write(|w| unsafe { w.pdr().bits(0) }); // Clear all P2 direction bits
    device.PORT0.podr().write(|w| unsafe { w.podr().bits(0) }); // Clear all P0 output bits
    device.PORT2.podr().write(|w| unsafe { w.podr().bits(0) }); // Clear all P2 output bits
    rprintln!("show_led_matrix {}", x + y * 12);
    if let (Ports::Port0xxPins(low), Ports::Port0xxPins(high)) = PIN_LIST[x + y * 12] {
        rprintln!("port0, port 0");
        device.PORT0.pdr().write(|w| unsafe {
            w.pdr().bits(low | high)
        }); // Set both as outputs
        device.PORT0.podr().write(|w| unsafe { w.podr().bits(high) }); 
    } else if let (Ports::Port0xxPins(low), Ports::Port2xxPins(high)) = PIN_LIST[x + y * 12] {
        rprintln!("port0, port 1");
        device.PORT0.pdr().write(|w| unsafe {
            w.pdr().bits(low)
        }); // Set both as outputs
        device.PORT2.pdr().write(|w| unsafe {
            w.pdr().bits(high)
        }); // Set both as outputs
        device.PORT2.podr().write(|w| unsafe { w.podr().bits(high) });
    } else if let (Ports::Port2xxPins(low), Ports::Port0xxPins(high)) = PIN_LIST[x + y * 12] {
        rprintln!("port1, port 0");
        device.PORT2.pdr().write(|w| unsafe {
            w.pdr().bits(low)
        }); // Set both as outputs
        device.PORT0.pdr().write(|w| unsafe {
            w.pdr().bits(high)
        }); // Set both as outputs
        device.PORT0.podr().write(|w| unsafe { w.podr().bits(high) }); 
    } else if let (Ports::Port2xxPins(low), Ports::Port2xxPins(high)) = PIN_LIST[x + y * 12] {
        rprintln!("port1, port 1");
        device.PORT2.pdr().write(|w| unsafe {
            w.pdr().bits(low | high)
        }); // Set both as outputs
        device.PORT2.podr().write(|w| unsafe { w.podr().bits(high) });
    }


}

fn draw_heart(device: &ra4m1::Peripherals)
{
    let heart: [[u8; 12]; 8] = [
        [ 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0 ],
        [ 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0 ],
        [ 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0 ],
        [ 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0 ],
        [ 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0 ],
        [ 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0 ],
        [ 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0 ],
        [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ]
    ];
    draw_pixel(&device, heart.map(|row| {
        row.map(|cell| cell != 0)
    }));
}



fn draw_right_arrow(device: &ra4m1::Peripherals)
{
    let heart: [[u8; 12]; 8] = [
        [0,0,0,0,0,0,0,0,1,0,0,0],
        [0,0,0,0,0,0,0,0,1,1,0,0],
        [0,0,0,0,0,0,0,0,1,1,1,0],
        [1,1,1,1,1,1,1,1,1,1,1,1],
        [1,1,1,1,1,1,1,1,1,1,1,1],
        [0,0,0,0,0,0,0,0,1,1,1,0],
        [0,0,0,0,0,0,0,0,1,1,0,0],
        [0,0,0,0,0,0,0,0,1,0,0,0]
    ];
    draw_pixel(&device, heart.map(|row| {
        row.map(|cell| cell != 0)
    }));
}


fn draw_left_arrow(device: &ra4m1::Peripherals)
{
    let heart: [[u8; 12]; 8] = [
        [0,0,0,1,0,0,0,0,0,0,0,0],
        [0,0,1,1,0,0,0,0,0,0,0,0],
        [0,1,1,1,0,0,0,0,0,0,0,0],
        [1,1,1,1,1,1,1,1,1,1,1,1],
        [1,1,1,1,1,1,1,1,1,1,1,1],
        [0,1,1,1,0,0,0,0,0,0,0,0],
        [0,0,1,1,0,0,0,0,0,0,0,0],
        [0,0,0,1,0,0,0,0,0,0,0,0]
    ];
    draw_pixel(&device, heart.map(|row| {
        row.map(|cell| cell != 0)
    }));
}

fn draw_pixel(device: &ra4m1::Peripherals, pixel:[[bool; 12]; 8])
{
    for y in 0..8 {
        for x in 0..12 {
            if pixel[y][x] {
                turnon_led_matrix(&device, x, y);
            }
        }
        device.PORT0.pdr().write(|w| unsafe { w.pdr().bits(0) }); // Set both as inputs
        device.PORT2.pdr().write(|w| unsafe { w.pdr().bits(0) });
    }
}


#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("RTT initialization done.");
    rprint!("Here goes a mandatory message: ");
    rprintln!("Hello, World!");
    //
    
    let device = unsafe { pac::Peripherals::steal() };
    //
    //loop {
    //    draw_heart(&device);
    //}

    let port0 = &device.PORT0;
    let port1 = &device.PORT1;

    let pfs = &device.PFS;

    let pmisc = &device.PMISC;

    let button_pin_mask12 = 1 << 12;
    let button_pin_mask11 = 1 << 11;
    let button_pin_mask07 = 1 << 7;
    let button_pin_mask06 = 1 << 6;

    // --- ボタンピン(P004)のセットアップ ---
    // 1. Port Direction Register (PDR) のビット4を'0'に設定し、入力モードにする
    port1.pdr().modify(|r, w| unsafe {
        w.bits(r.bits() & !button_pin_mask12)
    });
    // 2. Pull-up Control Register (PCR) のビット4を'1'に設定し、内部プルアップを有効にする

    pmisc.pwpr.write(|w|w.b0wi().clear_bit());
    pmisc.pwpr.write(|w|w.pfswe().set_bit()); // PFSレジスタの保護を解除
     pfs.p112pfs().modify(|_r, w| {
        w.pcr().set_bit() // PCR (Pull-up Control) ビットを1に設定
    });
    pmisc.pwpr.write(|w|w.pfswe().set_bit()); // PFSレジスタの保護を解除
     pfs.p111pfs().modify(|_r, w| {
        w.pcr().set_bit() // PCR (Pull-up Control) ビットを1に設定
    });
    pmisc.pwpr.write(|w|w.pfswe().clear_bit()); // PFSレジスタ保護
    pmisc.pwpr.write(|w|w.pfswe().set_bit());

    loop {
        let is_pressed12 = (port1.pidr().read().bits() & button_pin_mask12) == 0;
        let is_pressed11 = (port1.pidr().read().bits() & button_pin_mask11) == 0;

        // ボタンが押されていればLEDを点灯、押されていなければ消灯
        if is_pressed12 {
            draw_left_arrow(&device);
            rprintln!("flag 0");
        } 
        else if is_pressed11 
        {
            draw_right_arrow(&device);
            rprintln!("flag 1");
        }
        else {
            draw_heart(&device);
        }
    }

}

fn nop_delay(i:u32)
{
        for _ in 0..i {
            nop();
        }
}
