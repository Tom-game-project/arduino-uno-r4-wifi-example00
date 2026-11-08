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

/// 12×8 = 96ピクセルを96bit (12バイト) に隙間なく詰めたビットマップ型
/// bit (x + y * 12) が座標 (x, y) のON/OFFに対応する
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct MatrixBitmap(pub [u32; 3]);

impl MatrixBitmap {
    pub const EMPTY: Self = Self([0; 3]);

    /// 2進数リテラル8行（各行下位12bit、左端がx=0）からコンパイル時にビットマップを生成する
    #[inline(always)]
    pub const fn from_rows(rows: [u16; 8]) -> Self {
        let mut words = [0u32; 3];
        let mut y = 0;
        while y < 8 {
            let row = rows[y];
            let mut x = 0;
            while x < 12 {
                // 2進数表記の左端(bit 11)を x=0、右端(bit 0)を x=11 に対応させる
                if ((row >> (11 - x)) & 1) != 0 {
                    let idx = x + y * 12;
                    words[idx >> 5] |= 1u32 << (idx & 31);
                }
                x += 1;
            }
            y += 1;
        }
        Self(words)
    }

    /// 既存の [[bool; 12]; 8] 配列からビットマップへ変換する（const文脈でも実行時でも利用可能）
    #[inline(always)]
    pub const fn from_bools(pixel: &[[bool; 12]; 8]) -> Self {
        let mut words = [0u32; 3];
        let mut y = 0;
        while y < 8 {
            let mut x = 0;
            while x < 12 {
                if pixel[y][x] {
                    let idx = x + y * 12;
                    words[idx >> 5] |= 1u32 << (idx & 31);
                }
                x += 1;
            }
            y += 1;
        }
        Self(words)
    }

    /// 実行時に特定の座標 (x, y) のピクセルを書き換える
    #[inline(always)]
    pub fn set(&mut self, x: usize, y: usize, on: bool) {
        if x < 12 && y < 8 {
            let idx = x + y * 12;
            let mask = 1u32 << (idx & 31);
            if on {
                self.0[idx >> 5] |= mask;
            } else {
                self.0[idx >> 5] &= !mask;
            }
        }
    }
}

/// 下位16bit = Port0のピンマスク, 上位16bit = Port2のピンマスク
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct PinMask(u32);

impl PinMask {
    #[inline(always)]
    pub const fn port0(bit: u8) -> Self {
        Self(1u32 << bit)
    }

    #[inline(always)]
    pub const fn port2(bit: u8) -> Self {
        Self(1u32 << (bit + 16))
    }

    /// (low, high) のペアから、Port0とPort2それぞれの PCNTR1 レジスタ値
    /// (上位16bit = PODR, 下位16bit = PDR) を分岐なしで直接合成する
    #[inline(always)]
    pub const fn pcntr1_values(low: Self, high: Self) -> (u32, u32) {
        let pdr_both = low.0 | high.0;
        let p0_pcntr1 = (high.0 << 16) | (pdr_both & 0x0000_FFFF);
        let p2_pcntr1 = (high.0 & 0xFFFF_0000) | (pdr_both >> 16);
        (p0_pcntr1, p2_pcntr1)
    }
}

// Define pin bits for LED matrix control
// P0xx pins
const P003_BIT: PinMask = PinMask::port0(3);
const P004_BIT: PinMask = PinMask::port0(4);
const P011_BIT: PinMask = PinMask::port0(11);
const P012_BIT: PinMask = PinMask::port0(12);
const P013_BIT: PinMask = PinMask::port0(13);
const P015_BIT: PinMask = PinMask::port0(15);

// P2xx pins
const P204_BIT: PinMask = PinMask::port2(4);
const P205_BIT: PinMask = PinMask::port2(5);
const P206_BIT: PinMask = PinMask::port2(6);
const P212_BIT: PinMask = PinMask::port2(12);
const P213_BIT: PinMask = PinMask::port2(13);

const PINS: [PinMask; 11] = [
    P205_BIT, P012_BIT, P013_BIT, P206_BIT, P003_BIT, P204_BIT,
    P015_BIT, P004_BIT, P011_BIT, P213_BIT, P212_BIT,
];

#[inline(always)]
fn get_pins_math(idx: usize) -> (PinMask, PinMask) {
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
    #[inline(always)]
    pub fn drive_pair(self, pair: (PinMask, PinMask)) -> MatrixPortGuard<'a, Active> {
        let (low_pin, high_pin) = pair;
        let (p0_pcntr1, p2_pcntr1) = PinMask::pcntr1_values(low_pin, high_pin);

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

/// 1ピクセル（1次元インデックス 0..96）の点灯〜一定時間保持〜消灯までを型で強制する
#[inline(always)]
fn turnon_led_index<'a>(
    guard: MatrixPortGuard<'a, DirectionsCleared>,
    index: usize,
) -> MatrixPortGuard<'a, DirectionsCleared> {
    rprintln!("show_led_matrix {}", index);
    guard
        .clear_outputs()
        .drive_pair(get_pins_math(index))
        .hold_uniform()
        .turn_off()
}

/// ビットマップを受け取り、立っているビット（点灯ピクセル）だけを直接ジャンプ走査して描画する
pub fn draw_bitmap(device: &ra4m1::Peripherals, bitmap: &MatrixBitmap) {
    let mut guard: MatrixPortGuard<'_, DirectionsCleared> =
        MatrixPortGuard::new(device).clear_directions();

    for (word_idx, & initial_word) in bitmap.0.iter().enumerate() {
        let base_index = word_idx << 5; // word_idx * 32
        let mut word = initial_word;

        // 光っているビットが存在する間だけループ（消灯ピクセルは0サイクルでスキップ）
        while word != 0 {
            let bit_pos = word.trailing_zeros() as usize;
            guard = turnon_led_index(guard, base_index + bit_pos);

            // 最下位の「1」のビットだけをクリアして次の点灯ビットへ進む
            word &= word - 1;
        }
    }
}

/// 互換用: 既存の &[[bool; 12]; 8] からもそのまま描画可能
pub fn draw_pixel(device: &ra4m1::Peripherals, pixel: &[[bool; 12]; 8]) {
    let bitmap = MatrixBitmap::from_bools(pixel);
    draw_bitmap(device, &bitmap);
}

