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
struct PinMask(u32);

impl PinMask {
    #[inline(always)]
    pub const fn port0(bit: u8) -> Self {
        assert!(bit < 16);
        Self(1u32 << bit)
    }

    #[inline(always)]
    pub const fn port2(bit: u8) -> Self {
        assert!(bit < 16);
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
    P205_BIT, P012_BIT, P013_BIT, P206_BIT, P003_BIT, P204_BIT, P015_BIT, P004_BIT, P011_BIT,
    P213_BIT, P212_BIT,
];

#[inline(always)]
fn get_pins_math(idx: usize) -> (PinMask, PinMask) {
    assert!(idx < 96);
    let k = idx >> 1;
    // ビット 0, 1, 3, 6, 10, 15, 21, 28, 36, 45 が1の定数
    const TRI_MASK: u64 = 0x2010_1020_844B;

    // k以下の境界の数を数えるだけで i (1..=10) が求まる
    let i = (TRI_MASK & ((2u64 << k) - 1)).count_ones() as usize;
    let j = k - ((i * (i - 1)) >> 1);

    let (a, b) = (PINS[i], PINS[j]);
    if (idx & 1) == 0 {
        (a, b)
    } else {
        (b, a)
    }
}

// ============================================================================
// チャーリープレクシングLEDマトリクスの幽霊型（Typestate）ステートマシン
// ============================================================================

struct Unreset;
struct DirectionsCleared;
struct ReadyToDrive;
struct Active;
struct HoldCompleted;

/// ゼロコストなPORT0/PORT2レジスタ操作ステートマシン
struct MatrixPortGuard<'a, State> {
    port0: &'a mut ra4m1::PORT0,
    port2: &'a mut ra4m1::PORT2,
    _state: PhantomData<State>,
}

impl<'a> MatrixPortGuard<'a, Unreset> {
    #[inline(always)]
    fn new(device: &'a mut LedMatrix) -> Self {
        Self {
            port0: &mut device.port0,
            port2: &mut device.port2,
            _state: PhantomData,
        }
    }

    #[inline(always)]
    pub fn clear_directions(self) -> MatrixPortGuard<'a, DirectionsCleared> {
        // SAFETY: LED用ピンのみをクリアし、その他のピンを保持する。
        self.port0
            .pdr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !PORT0_MASK) });
        // SAFETY: LED用ピンのみをクリアし、その他のピンを保持する。
        self.port2
            .pdr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !PORT2_MASK) });
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
        // SAFETY: LED用ピンのみをクリアし、その他のピンを保持する。
        self.port0
            .podr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !PORT0_MASK) });
        // SAFETY: LED用ピンのみをクリアし、その他のピンを保持する。
        self.port2
            .podr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !PORT2_MASK) });
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

        // SAFETY: 検証済みのLEDペアだけを駆動し、対象外のPDR/PODRを保持する。
        self.port0
            .pcntr1()
            .modify(|r, w| unsafe { w.bits(merge_pcntr1(r.bits(), p0_pcntr1, PORT0_MASK)) });
        // SAFETY: 検証済みのLEDペアだけを駆動し、対象外のPDR/PODRを保持する。
        self.port2
            .pcntr1()
            .modify(|r, w| unsafe { w.bits(merge_pcntr1(r.bits(), p2_pcntr1, PORT2_MASK)) });

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
        // SAFETY: LED用ピンのみをクリアし、その他のピンを保持する。
        self.port0
            .pdr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !PORT0_MASK) });
        // SAFETY: LED用ピンのみをクリアし、その他のピンを保持する。
        self.port2
            .pdr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !PORT2_MASK) });
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

const PORT0_MASK: u16 = (1 << 3) | (1 << 4) | (1 << 11) | (1 << 12) | (1 << 13) | (1 << 15);
const PORT2_MASK: u16 = (1 << 4) | (1 << 5) | (1 << 6) | (1 << 12) | (1 << 13);

fn merge_pcntr1(old: u32, value: u32, pins: u16) -> u32 {
    let mask = u32::from(pins) | (u32::from(pins) << 16);
    (old & !mask) | (value & mask)
}

/// PORT0/PORT2の所有権を保持し、同時に複数の駆動状態を作れなくする。
///
/// レジスタを書き換える描画には排他参照が必要。
/// ```compile_fail,E0596
/// use arduino_uno_r4_wifi_example00::led_matrix::{LedMatrix, MatrixBitmap};
/// fn shared_draw(matrix: &LedMatrix) {
///     matrix.draw_bitmap(&MatrixBitmap::EMPTY);
/// }
/// ```
///
/// 所有しているポートを外部から取得して状態を変更することはできない。
/// ```compile_fail,E0616
/// use arduino_uno_r4_wifi_example00::led_matrix::LedMatrix;
/// fn bypass(matrix: &LedMatrix) {
///     matrix.port0.pdr().reset();
/// }
/// ```
pub struct LedMatrix {
    port0: ra4m1::PORT0,
    port2: ra4m1::PORT2,
}

impl LedMatrix {
    pub(crate) fn new(port0: ra4m1::PORT0, port2: ra4m1::PORT2) -> Self {
        let mut matrix = Self { port0, port2 };
        MatrixPortGuard::new(&mut matrix)
            .clear_directions()
            .clear_outputs();
        matrix
    }

    /// 1フレームを走査し、終了時は必ず全LEDピンを入力に戻す。
    pub fn draw_bitmap(&mut self, bitmap: &MatrixBitmap) {
        let mut guard = MatrixPortGuard::new(self).clear_directions();
        for (word_idx, &initial_word) in bitmap.0.iter().enumerate() {
            let base_index = word_idx << 5;
            let mut word = initial_word;
            while word != 0 {
                let bit_pos = word.trailing_zeros() as usize;
                guard = turnon_led_index(guard, base_index + bit_pos);
                word &= word - 1;
            }
        }
    }

    pub fn draw_pixel(&mut self, pixel: &[[bool; 12]; 8]) {
        self.draw_bitmap(&MatrixBitmap::from_bools(pixel));
    }
}

impl Drop for LedMatrix {
    fn drop(&mut self) {
        MatrixPortGuard::new(self).clear_directions();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_pixels_select_distinct_unique_directed_pairs() {
        let mut seen = [[false; 11]; 11];
        // Independent reference: enumerate each triangular group in wiring order.
        let mut index = 0;
        for i in 1..11 {
            for j in 0..i {
                for (low, high) in [(i, j), (j, i)] {
                    if index == 96 {
                        return;
                    }
                    let pair = get_pins_math(index);
                    assert_eq!(pair, (PINS[low], PINS[high]));
                    assert_ne!(pair.0, pair.1);
                    assert!(!seen[low][high]);
                    seen[low][high] = true;
                    let (p0, p2) = PinMask::pcntr1_values(pair.0, pair.1);
                    assert_eq!((p0 & 0xffff).count_ones() + (p2 & 0xffff).count_ones(), 2);
                    assert_eq!((p0 >> 16).count_ones() + (p2 >> 16).count_ones(), 1);
                    index += 1;
                }
            }
        }
        panic!("not all pixels were checked");
    }

    #[test]
    fn register_updates_preserve_every_unrelated_pin() {
        for index in 0..96 {
            let (low, high) = get_pins_math(index);
            let (p0, p2) = PinMask::pcntr1_values(low, high);
            for (value, pins) in [(p0, PORT0_MASK), (p2, PORT2_MASK)] {
                let mask = u32::from(pins) * 0x0001_0001;
                for old in [0, u32::MAX, 0xaaaa_5555, 0x5555_aaaa] {
                    let merged = merge_pcntr1(old, value, pins);
                    assert_eq!(merged & !mask, old & !mask);
                    assert_eq!(merged & mask, value);
                }
            }
        }
    }

    #[test]
    fn bitmap_coordinates_cover_word_boundaries_and_row_orientation() {
        for y in 0..8 {
            for x in 0..12 {
                let mut bitmap = MatrixBitmap::EMPTY;
                bitmap.set(x, y, true);
                let mut bools = [[false; 12]; 8];
                bools[y][x] = true;
                let mut rows = [0; 8];
                rows[y] = 1 << (11 - x);
                assert_eq!(bitmap, MatrixBitmap::from_bools(&bools));
                assert_eq!(bitmap, MatrixBitmap::from_rows(rows));
                assert_eq!(bitmap.0.iter().map(|w| w.count_ones()).sum::<u32>(), 1);
                bitmap.set(x, y, false);
                assert_eq!(bitmap, MatrixBitmap::EMPTY);
            }
        }
        let mut bitmap = MatrixBitmap::EMPTY;
        bitmap.set(usize::MAX, 0, true);
        bitmap.set(0, usize::MAX, true);
        assert_eq!(bitmap, MatrixBitmap::EMPTY);
    }

    #[test]
    #[should_panic]
    fn port0_rejects_out_of_range_bit() {
        let _ = PinMask::port0(16);
    }

    #[test]
    #[should_panic]
    fn port2_rejects_out_of_range_bit() {
        let _ = PinMask::port2(16);
    }
}
