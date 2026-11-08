// ```
// cargo run --release
// # With rtt output, it slows down and the LED flickers, so it is for testing purposes.
// cargo run --release --features=rtt
// # Temporarily allow writing to the connected microcomputer device.
// sudo chmod a+rw /dev/hidraw*
// ```
#![no_main]
#![no_std]

use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};
use ra4m1 as pac;
mod led_matrix;
mod sci;
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
    loop {}
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

// ============================================================================
// 1. PWPR（PFS書き込みプロテクトレジスタ）の幽霊型（Typestate）ステートマシン
//    要求されるハードウェア操作順序:
//    [Locked] --(1. B0WI=0)--> [B0wiCleared] --(2. PFSWE=1)--> [Unlocked (PFS操作可)]
//             --(3. PFSWE=0)--> [PfsweCleared] --(4. B0WI=1)--> [Locked]
// ============================================================================

pub struct Locked;
pub struct B0wiCleared;
pub struct Unlocked;
pub struct PfsweCleared;

/// ゼロコストなPWPRレジスタの型状態ラッパー（実行時サイズ = `&pac::PMISC` と同一）
#[repr(transparent)]
pub struct PwprGuard<'a, State> {
    pmisc: &'a pac::PMISC,
    _state: PhantomData<State>,
}

impl<'a> PwprGuard<'a, Locked> {
    #[inline(always)]
    pub fn new(pmisc: &'a pac::PMISC) -> Self {
        Self {
            pmisc,
            _state: PhantomData,
        }
    }

    /// 手順1: B0WIビットを0にクリアし、PFSWEビットへの書き込みを許可する状態へ遷移
    #[inline(always)]
    pub fn clear_b0wi(self) -> PwprGuard<'a, B0wiCleared> {
        self.pmisc.pwpr.write(|w| w.b0wi().clear_bit());
        PwprGuard {
            pmisc: self.pmisc,
            _state: PhantomData,
        }
    }
}

impl<'a> PwprGuard<'a, B0wiCleared> {
    /// 手順2: B0WIがクリアされた状態でのみPFSWEビットを1にセットでき、PFS保護解除状態へ遷移
    #[inline(always)]
    pub fn enable_pfswe(self) -> PwprGuard<'a, Unlocked> {
        self.pmisc.pwpr.write(|w| w.pfswe().set_bit());
        PwprGuard {
            pmisc: self.pmisc,
            _state: PhantomData,
        }
    }
}

impl<'a> PwprGuard<'a, Unlocked> {
    /// 手順3: PFSの設定完了後、まずPFSWEビットを0にクリアする状態へ遷移
    #[inline(always)]
    pub fn disable_pfswe(self) -> PwprGuard<'a, PfsweCleared> {
        self.pmisc.pwpr.write(|w| w.pfswe().clear_bit());
        PwprGuard {
            pmisc: self.pmisc,
            _state: PhantomData,
        }
    }
}

impl<'a> PwprGuard<'a, PfsweCleared> {
    /// 手順4: PFSWEをクリアした後にB0WIビットを1に戻し、確実にLocked状態へ復帰
    /// （元のコードにあった `pfswe().set_bit()` を誤って再度呼んでしまうバグを型レベルで防止）
    #[inline(always)]
    pub fn lock_b0wi(self) -> PwprGuard<'a, Locked> {
        self.pmisc.pwpr.write(|w| w.b0wi().set_bit());
        PwprGuard {
            pmisc: self.pmisc,
            _state: PhantomData,
        }
    }
}

// ============================================================================
// 2. PORT1 ボタン入力ピンの幽霊型（Typestate）ステートマシン
//    要求されるハードウェア操作順序:
//    [Unconfigured] --(1. PDRを入力=0に設定)--> [InputMode]
//                   --(2. PwprGuard<Unlocked>証明付きでPFS.PCR=1に設定)--> [PullUpReady]
//                   --(3. PIDR読み取り可能)--> ボタン判定
// ============================================================================

const BUTTON_PIN_MASK12: u16 = 1 << 12;
const BUTTON_PIN_MASK11: u16 = 1 << 11;
const BUTTON_PIN_MASK07: u16 = 1 << 7;
const BUTTON_PIN_MASK06: u16 = 1 << 6;
const ALL_BUTTON_MASK: u16 =
    BUTTON_PIN_MASK12 | BUTTON_PIN_MASK11 | BUTTON_PIN_MASK07 | BUTTON_PIN_MASK06;

pub struct Unconfigured;
pub struct InputMode;
pub struct PullUpReady;

pub struct ButtonStates {
    pub is_pressed12: bool,
    pub is_pressed11: bool,
    pub is_pressed07: bool,
    pub is_pressed06: bool,
}

/// ゼロコストなPORT1ボタン群の型状態ラッパー
#[repr(transparent)]
pub struct Port1Buttons<'a, State> {
    port1: &'a pac::PORT1,
    _state: PhantomData<State>,
}

impl<'a> Port1Buttons<'a, Unconfigured> {
    #[inline(always)]
    pub fn new(port1: &'a pac::PORT1) -> Self {
        Self {
            port1,
            _state: PhantomData,
        }
    }

    /// 手順1: Port Direction Register (PDR) の対象ビットを0にし、入力モードへ遷移
    #[inline(always)]
    pub fn into_input_mode(self) -> Port1Buttons<'a, InputMode> {
        // SAFETY: ボタン用ピンマスクの方向ビットのみを0（入力）にクリアします。
        self.port1
            .pdr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !ALL_BUTTON_MASK) });
        Port1Buttons {
            port1: self.port1,
            _state: PhantomData,
        }
    }
}

impl<'a> Port1Buttons<'a, InputMode> {
    /// 手順2: 入力モード設定済み かつ PWPRが `Unlocked` 状態のときのみ、内部プルアップ(PCR)を有効化可能
    #[inline(always)]
    pub fn enable_pullups(
        self,
        pfs: &pac::PFS,
        _pwpr_unlocked: &PwprGuard<'_, Unlocked>,
    ) -> Port1Buttons<'a, PullUpReady> {
        pfs.p112pfs().modify(|_r, w| w.pcr().set_bit());
        pfs.p111pfs().modify(|_r, w| w.pcr().set_bit());
        pfs.p107pfs_ha().modify(|_r, w| w.pcr().set_bit());
        pfs.p106pfs_ha().modify(|_r, w| w.pcr().set_bit());
        Port1Buttons {
            port1: self.port1,
            _state: PhantomData,
        }
    }
}

impl<'a> Port1Buttons<'a, PullUpReady> {
    /// 手順3: 入力方向とプルアップ設定が完了した `PullUpReady` 状態でのみ PIDR の読み取りを許可
    #[inline(always)]
    pub fn read(&self) -> ButtonStates {
        let bits = self.port1.pidr().read().bits();
        ButtonStates {
            is_pressed12: (bits & BUTTON_PIN_MASK12) == 0,
            is_pressed11: (bits & BUTTON_PIN_MASK11) == 0,
            is_pressed07: (bits & BUTTON_PIN_MASK07) == 0,
            is_pressed06: (bits & BUTTON_PIN_MASK06) == 0,
        }
    }
}

fn draw_heart(device: &ra4m1::Peripherals) {
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
    led_matrix::draw_pixel(device, &heart.map(|row| row.map(|cell| cell != 0)));
}

fn draw_right_arrow(device: &ra4m1::Peripherals) {
    let heart: [[u8; 12]; 8] = [
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0],
    ];
    led_matrix::draw_pixel(device, &heart.map(|row| row.map(|cell| cell != 0)));
}

fn draw_left_arrow(device: &ra4m1::Peripherals) {
    let heart: [[u8; 12]; 8] = [
        [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    led_matrix::draw_pixel(device, &heart.map(|row| row.map(|cell| cell != 0)));
}

fn draw_right_left_arrow(device: &ra4m1::Peripherals) {
    let heart: [[u8; 12]; 8] = [
        [0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0],
        [0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0],
        [0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 0],
        [0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0],
        [0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0],
    ];
    led_matrix::draw_pixel(device, &heart.map(|row| row.map(|cell| cell != 0)));
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

    // 幽霊型（Typestate）により、ハードウェアが要求する設定順序をコンパイル時に強制（実行時コスト0）
    // 1. PORT1のボタンピンを入力モードへ遷移
    let buttons_input = Port1Buttons::new(&device.PORT1).into_input_mode();

    // 2. PWPRを [Locked] -> [B0wiCleared] -> [Unlocked] の順で遷移させてPFS保護を解除
    let pwpr_unlocked = PwprGuard::new(&device.PMISC).clear_b0wi().enable_pfswe();

    // 3. `InputMode` かつ `PwprGuard<Unlocked>` の証明がある状態でのみプルアップを有効化
    let buttons = buttons_input.enable_pullups(&device.PFS, &pwpr_unlocked);

    // 4. PWPRを [Unlocked] -> [PfsweCleared] -> [Locked] の順で確実に再保護
    let _pwpr_locked: PwprGuard<'_, Locked> = pwpr_unlocked.disable_pfswe().lock_b0wi();

    loop {
        // `PullUpReady` 状態の `buttons` からのみ安全に状態を読み取れる
        let state = buttons.read();

        if state.is_pressed11 && state.is_pressed12 {
            draw_right_left_arrow(&device);
        } else if state.is_pressed12 || state.is_pressed07 {
            draw_left_arrow(&device);
            rprintln!("flag 0");
        } else if state.is_pressed11 || state.is_pressed06 {
            draw_right_arrow(&device);
            rprintln!("flag 1");
        } else {
            draw_heart(&device);
        }
    }
}

#[allow(dead_code)]
fn nop_delay(i: u32) {
    for _ in 0..i {
        nop();
    }
}
