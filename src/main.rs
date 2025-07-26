// ```
// cargo run --release
// # With rtt output, it slows down and the LED flickers, so it is for testing purposes.
// cargo run --release --features=rtt
// # Temporarily allow writing to the connected microcomputer device.
// sudo chmod a+rw /dev/hidraw*
// ```
#![no_main]
#![no_std]

use ra4m1 as pac;
mod setting;
mod sci;
mod led_matrix;

// ===== rtt debug tools =====
#[cfg(feature = "rtt")]
use panic_rtt_target as _;
#[cfg(feature="rtt")]
use rtt_target::{rprint, rprintln, rtt_init_print};
#[cfg(not(feature="rtt"))]
macro_rules! rtt_init_print { () => { } }
#[cfg(not(feature="rtt"))]
macro_rules! rprint { ($($arg:tt)*) => { let _ = ($($arg)*); } }
#[cfg(not(feature="rtt"))]
macro_rules! rprintln { ($($arg:tt)*) => { let _ = ($($arg)*); } }
// ===== rtt debug tools =====

#[allow(unused)]
use cortex_m::asm::nop;

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

fn draw_heart(device: &ra4m1::Peripherals)
{
    let heart: [[u8; 12]; 8] = [
        [0,0,1,1,0,0,0,1,1,0,0,0],
        [0,1,1,1,1,0,1,1,1,1,0,0],
        [0,1,1,1,1,1,1,1,1,1,0,0],
        [0,0,1,1,1,1,1,1,1,0,0,0],
        [0,0,0,1,1,1,1,1,0,0,0,0],
        [0,0,0,0,1,1,1,0,0,0,0,0],
        [0,0,0,0,0,1,0,0,0,0,0,0],
        [0,0,0,0,0,0,0,0,0,0,0,0]
    ];
        led_matrix::draw_pixel(&device, heart.map(|row| {
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
    led_matrix::draw_pixel(&device, heart.map(|row| {
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
    led_matrix::draw_pixel(&device, heart.map(|row| {
        row.map(|cell| cell != 0)
    }));
}

fn draw_right_left_arrow(device: &ra4m1::Peripherals)
{
    let heart: [[u8; 12]; 8] = [
        [0,0,0,1,0,0,0,0,1,0,0,0],
        [0,0,1,1,0,0,0,0,1,1,0,0],
        [0,1,1,1,0,0,0,0,1,1,1,0],
        [1,1,1,1,1,1,1,1,1,1,1,1],
        [1,1,1,1,1,1,1,1,1,1,1,1],
        [0,1,1,1,0,0,0,0,1,1,1,0],
        [0,0,1,1,0,0,0,0,1,1,0,0],
        [0,0,0,1,0,0,0,0,1,0,0,0]
    ];
    led_matrix::draw_pixel(&device, heart.map(|row| {
        row.map(|cell| cell != 0)
    }));
}

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("RTT initialization done.");
    rprint!("Here goes a mandatory message: ");
    rprintln!("Hello, World!");

    let device = unsafe { pac::Peripherals::steal() };

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
    pfs.p111pfs().modify(|_r, w| {
        w.pcr().set_bit() // PCR (Pull-up Control) ビットを1に設定
    });
    pfs.p107pfs_ha().modify(|_r, w| {
        w.pcr().set_bit() // PCR (Pull-up Control) ビットを1に設定
    });
    pfs.p106pfs_ha().modify(|_r, w| {
        w.pcr().set_bit() // PCR (Pull-up Control) ビットを1に設定
    });
    pmisc.pwpr.write(|w|w.pfswe().clear_bit()); // PFSレジスタ保護
    pmisc.pwpr.write(|w|w.pfswe().set_bit());

    loop {
        let is_pressed12 = (port1.pidr().read().bits() & button_pin_mask12) == 0;
        let is_pressed11 = (port1.pidr().read().bits() & button_pin_mask11) == 0;
        let is_pressed07 = (port1.pidr().read().bits() & button_pin_mask07) == 0;
        let is_pressed06 = (port1.pidr().read().bits() & button_pin_mask06) == 0;

        // ボタンが押されていればLEDを点灯、押されていなければ消灯
        if is_pressed11 && is_pressed12 {
            draw_right_left_arrow(&device);
        }
        else if is_pressed12 || is_pressed07{
            draw_left_arrow(&device);
            rprintln!("flag 0");
        } 
        else if is_pressed11 || is_pressed06
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
