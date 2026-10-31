// LED MATRIX CODE

use core::marker::PhantomData;
use cortex_m::asm::nop;

// ===== rtt debug tools =====
#[cfg(feature = "rtt")]
use panic_rtt_target as _;
#[cfg(feature = "rtt")]
use rtt_target::rprintln;
#[cfg(not(feature = "rtt"))]
macro_rules! rprintln {
    ($($arg:tt)*) => {
        let _ = ($($arg)*);
    };
}
// ===== rtt debug tools =====

/// 各点灯ピクセルが均一に光り続けるホールドサイクル数
const PIXEL_HOLD_CYCLES: u32 = 32;

#[derive(Copy, Clone)]
enum Ports {
    Port0xxPins(u16),
    Port2xxPins(u16),
}

// #[derive(Copy, Clone)]
// struct NewPorts(u32);
// 
// impl NewPorts {
//     pub const fn port0(bit: u8) -> Self { Self(1u32 << bit) }
//     pub const fn port2(bit: u8) -> Self { Self(1u32 << (bit + 16)) }
// 
//     #[inline(always)]
//     pub const fn split(self) -> (u16, u16) {
//         (self.0 as u16, (self.0 >> 16) as u16)
//     }
// }

impl Ports {
    /// (Port0のビットマスク, Port2のビットマスク) に分解する（全域性保証）
    #[inline(always)]
    pub const fn into_port_masks(self) -> (u16, u16) {
        match self {
            Ports::Port0xxPins(bits) => (bits, 0),
            Ports::Port2xxPins(bits) => (0, bits),
        }
    }
}

// Define pin bits for LED matrix control
// P0xx pins
const P003_BIT: Ports = Ports::Port0xxPins(1 << 3);
const P004_BIT: Ports = Ports::Port0xxPins(1 << 4);
const P011_BIT: Ports = Ports::Port0xxPins(1 << 11);
const P012_BIT: Ports = Ports::Port0xxPins(1 << 12);
const P013_BIT: Ports = Ports::Port0xxPins(1 << 13);
const P015_BIT: Ports = Ports::Port0xxPins(1 << 15);

// P2xx pins
const P204_BIT: Ports = Ports::Port2xxPins(1 << 4);
const P205_BIT: Ports = Ports::Port2xxPins(1 << 5);
const P206_BIT: Ports = Ports::Port2xxPins(1 << 6);
const P212_BIT: Ports = Ports::Port2xxPins(1 << 12);
const P213_BIT: Ports = Ports::Port2xxPins(1 << 13);

const PINS: [Ports; 11] = [
    P205_BIT, P012_BIT, P013_BIT, P206_BIT, P003_BIT, P204_BIT,
    P015_BIT, P004_BIT, P011_BIT, P213_BIT, P212_BIT,
];

#[inline(always)]
fn get_pins_math(idx: usize) -> (Ports, Ports) {
    let k = idx >> 1;
    // ビット 0, 1, 3, 6, 10, 15, 21, 28, 36, 45 が1の定数
    const TRI_MASK: u64 = 0x2010_1020_844B;

    // k以下の境界の数を数えるだけで i (1..=10) が求まる
    let i = (TRI_MASK & ((2u64 << k) - 1)).count_ones() as usize;
    let j = k - ((i * (i - 1)) >> 1);

    let (a, b) = (PINS[i], PINS[j]);
    if (idx & 1) == 0 { (a, b) } else { (b, a) }
}

// ============================================================================
// チャーリープレクシングLEDマトリクスの幽霊型（Typestate）ステートマシン
// 要求されるハードウェア操作順序:
// [Unreset]
//   --(1. clear_directions: PORT0.PDR=0, PORT2.PDR=0 で全ピンを入力Hi-Z化)--> [DirectionsCleared]
//   --(2. clear_outputs:    PORT0.PODR=0, PORT2.PODR=0 で出力ラッチを0初期化)--> [ReadyToDrive]
//   --(3. drive_pair:       対象ピンのPDR出力化・PODRのHigh出力設定)--> [Active]
//   --(4. hold_uniform:     全点灯ピクセル共通の一定時間だけ点灯を維持)--> [HoldCompleted]
//   --(5. turn_off:         次の列の走査へ移る前にPDR=0で確実に消灯)--> [DirectionsCleared]
// ============================================================================

pub struct Unreset;
pub struct DirectionsCleared;
pub struct ReadyToDrive;
pub struct Active;
pub struct HoldCompleted;

/// ゼロコストなPORT0/PORT2レジスタ操作ステートマシン
pub struct MatrixPortGuard<'a, State> {
    port0: &'a ra4m1::PORT0,
    port2: &'a ra4m1::PORT2,
    _state: PhantomData<State>,
}

impl<'a> MatrixPortGuard<'a, Unreset> {
    #[inline(always)]
    pub fn new(device: &'a ra4m1::Peripherals) -> Self {
        Self {
            port0: &device.PORT0,
            port2: &device.PORT2,
            _state: PhantomData,
        }
    }

    /// 手順1: まずPORT0 / PORT2の全方向ビット(PDR)を0（入力・Hi-Z）にクリアする
    #[inline(always)]
    pub fn clear_directions(self) -> MatrixPortGuard<'a, DirectionsCleared> {
        self.port0.pdr().write(|w| unsafe { w.pdr().bits(0) });
        self.port2.pdr().write(|w| unsafe { w.pdr().bits(0) });
        MatrixPortGuard {
            port0: self.port0,
            port2: self.port2,
            _state: PhantomData,
        }
    }
}

impl<'a> MatrixPortGuard<'a, DirectionsCleared> {
    /// 手順2: 方向ビット(PDR)が0にクリアされた状態でのみ、出力データレジスタ(PODR)を0にクリアする
    #[inline(always)]
    pub fn clear_outputs(self) -> MatrixPortGuard<'a, ReadyToDrive> {
        self.port0.podr().write(|w| unsafe { w.podr().bits(0) });
        self.port2.podr().write(|w| unsafe { w.podr().bits(0) });
        MatrixPortGuard {
            port0: self.port0,
            port2: self.port2,
            _state: PhantomData,
        }
    }
}

impl<'a> MatrixPortGuard<'a, ReadyToDrive> {
    /// 手順3: PDR=0 かつ PODR=0 が保証された `ReadyToDrive` 状態でのみ、指定ペアのPDR出力設定とPODR High設定を実行可能
    #[inline(always)]
    pub fn drive_pair(self, pair: (Ports, Ports)) -> MatrixPortGuard<'a, Active> {
        let (low_pin, high_pin) = pair;
        let (p0_low, p2_low) = low_pin.into_port_masks();
        let (p0_high, p2_high) = high_pin.into_port_masks();

        // 上位16bitがPODR(出力値)、下位16bitがPDR(方向: 1=出力)
        let p0_pcntr1 = ((p0_high as u32) << 16) | ((p0_low | p0_high) as u32);
        let p2_pcntr1 = ((p2_high as u32) << 16) | ((p2_low | p2_high) as u32);

        self.port0.pcntr1().write(|w| unsafe { w.bits(p0_pcntr1) });
        self.port2.pcntr1().write(|w| unsafe { w.bits(p2_pcntr1) });

        MatrixPortGuard {
            port0: self.port0,
            port2: self.port2,
            _state: PhantomData,
        }
    }
}

impl<'a> MatrixPortGuard<'a, Active> {
    /// 手順4: 点灯状態(Active)を一定サイクル保持し、列の位置による点灯時間の偏りをなくす
    #[inline(always)]
    pub fn hold_uniform(self) -> MatrixPortGuard<'a, HoldCompleted> {
        for _ in 0..PIXEL_HOLD_CYCLES {
            nop();
        }
        MatrixPortGuard {
            port0: self.port0,
            port2: self.port2,
            _state: PhantomData,
        }
    }
}

impl<'a> MatrixPortGuard<'a, HoldCompleted> {
    /// 手順5: 一定時間保持した直後に必ずPDR=0で消灯し、右側の消灯マス(false)走査中に光りっぱなしになるのを防ぐ
    #[inline(always)]
    pub fn turn_off(self) -> MatrixPortGuard<'a, DirectionsCleared> {
        self.port0.pdr().write(|w| unsafe { w.pdr().bits(0) });
        self.port2.pdr().write(|w| unsafe { w.pdr().bits(0) });
        MatrixPortGuard {
            port0: self.port0,
            port2: self.port2,
            _state: PhantomData,
        }
    }
}

/// 1ピクセルの点灯〜一定時間保持〜消灯(DirectionsCleared)までを1セットとして型で強制する
#[inline(always)]
fn turnon_led_matrix<'a>(
    guard: MatrixPortGuard<'a, DirectionsCleared>,
    x: usize,
    y: usize,
) -> MatrixPortGuard<'a, DirectionsCleared> {
    let index = x + y * 12;
    rprintln!("show_led_matrix {}", index);
    // [DirectionsCleared] -> (clear_outputs) -> [ReadyToDrive] -> (drive_pair)
    // -> [Active] -> (hold_uniform) -> [HoldCompleted] -> (turn_off) -> [DirectionsCleared]
    guard
        .clear_outputs()
        .drive_pair(get_pins_math(index))
        .hold_uniform()
        .turn_off()
}

pub fn draw_pixel(device: &ra4m1::Peripherals, pixel: [[bool; 12]; 8]) {
    // 最初に1回 `DirectionsCleared` (PDR=0) 状態を作り、そのガードを全ピクセル走査で引き回す
    let mut guard: MatrixPortGuard<'_, DirectionsCleared> =
        MatrixPortGuard::new(device).clear_directions();

    for y in 0..8 {
        for x in 0..12 {
            if pixel[y][x] {
                guard = turnon_led_matrix(guard, x, y);
            }
        }
    }
}
