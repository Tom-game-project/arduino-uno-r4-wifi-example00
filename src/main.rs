// ```
// cargo run --release
// # With rtt output, it slows down and the LED flickers, so it is for testing purposes.
// cargo run --release --features=rtt
// # Temporarily allow writing to the connected microcomputer device.
// sudo chmod a+rw /dev/hidraw*
// ```
#![no_main]
#![no_std]

use arduino_uno_r4_wifi_example00::{configure, led_matrix};
use core::sync::atomic::{AtomicBool, Ordering};
use ra4m1 as pac;
mod setting;

// ===== rtt debug tools =====
#[cfg(feature = "rtt")]
use panic_rtt_target as _;
#[cfg(feature = "rtt")]
use rtt_target::{rprint, rprintln, rtt_init_print};
#[cfg(not(feature = "rtt"))]
macro_rules! rtt_init_print {
    () => {};
}
#[cfg(not(feature = "rtt"))]
macro_rules! rprint {
    ($($arg:tt)*) => {
        let _ = ($($arg)*);
    };
}
#[cfg(not(feature = "rtt"))]
macro_rules! rprintln {
    ($($arg:tt)*) => {
        let _ = ($($arg)*);
    };
}
// ===== rtt debug tools =====

#[cfg(not(feature = "rtt"))]
#[inline(never)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::asm::udf();
}

#[inline(never)]
pub fn exit() -> ! {
    cortex_m::asm::bkpt();
    loop {
        cortex_m::asm::wfi();
    }
}

/// `ra4m1` クレートの標準設定では `Peripherals::take()` が生成されないため、
/// `AtomicBool` を用いてプログラム全体で1度だけ安全に `Peripherals` の所有権を取得するシングルトン関数
#[inline(always)]
fn take_peripherals() -> Option<pac::Peripherals> {
    static TAKEN: AtomicBool = AtomicBool::new(false);
    if !TAKEN.swap(true, Ordering::AcqRel) {
        // SAFETY: `TAKEN` によるアトミックな排他チェックにより、2回以上の取得（エイリアシング）が
        // 起きないことが保証されているため、この1回限りの `steal()` 呼び出しは安全です。
        Some(unsafe { pac::Peripherals::steal() })
    } else {
        None
    }
}

fn draw_heart(matrix: &mut led_matrix::LedMatrix) {
    let heart: [[u8; 12]; 8] = [
        [0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0],
        [0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0],
        [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
        [0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        [0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0],
        [0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    matrix.draw_pixel(&heart.map(|row| row.map(|cell| cell != 0)));
}

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("RTT initialization done.");
    rprint!("Here goes a mandatory message: ");
    rprintln!("Hello, World!");

    let Some(device) = take_peripherals() else {
        exit();
    };

    let mut matrix = configure(device);

    loop {
        draw_heart(&mut matrix);
    }
}
