use core::marker::PhantomData;
use ra4m1 as pac;

// ============================================================================
// 1. PWPR（PFS書き込みプロテクトレジスタ）の幽霊型（Typestate）ステートマシン
//    要求されるハードウェア操作順序:
//    [Locked] --(1. B0WI=0)--> [B0wiCleared] --(2. PFSWE=1)--> [Unlocked (PFS操作可)]
//             --(3. PFSWE=0)--> [PfsweCleared] --(4. B0WI=1)--> [Locked]
// ============================================================================

struct Locked;
struct B0wiCleared;
struct Unlocked;
struct PfsweCleared;

/// ゼロコストなPWPRレジスタの型状態ラッパー（実行時サイズ = `&pac::PMISC` と同一）
#[repr(transparent)]
struct PwprGuard<'a, State> {
    pmisc: &'a mut pac::PMISC,
    _state: PhantomData<State>,
}

impl<'a> PwprGuard<'a, Locked> {
    #[inline(always)]
    fn new(pmisc: &'a mut pac::PMISC) -> Self {
        Self {
            pmisc,
            _state: PhantomData,
        }
    }

    /// 手順1: B0WIビットを0にクリアし、PFSWEビットへの書き込みを許可する状態へ遷移
    #[inline(always)]
    fn clear_b0wi(self) -> PwprGuard<'a, B0wiCleared> {
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
    fn enable_pfswe(self) -> PwprGuard<'a, Unlocked> {
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
    fn disable_pfswe(self) -> PwprGuard<'a, PfsweCleared> {
        self.pmisc.pwpr.write(|w| w.pfswe().clear_bit());
        PwprGuard {
            pmisc: self.pmisc,
            _state: PhantomData,
        }
    }
}

impl<'a> PwprGuard<'a, PfsweCleared> {
    /// 手順4: PFSWEをクリアした後にB0WIビットを1に戻し、確実にLocked状態へ復帰
    #[inline(always)]
    fn lock_b0wi(self) -> PwprGuard<'a, Locked> {
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

struct Unconfigured;
struct InputMode;
pub struct PullUpReady;

pub struct ButtonStates {
    pub is_pressed12: bool,
    pub is_pressed11: bool,
    pub is_pressed07: bool,
    pub is_pressed06: bool,
}

/// ゼロコストなPORT1ボタン群の型状態ラッパー
#[repr(transparent)]
pub struct Port1Buttons<State> {
    port1: pac::PORT1,
    _state: PhantomData<State>,
}

impl Port1Buttons<Unconfigured> {
    #[inline(always)]
    fn new(port1: pac::PORT1) -> Self {
        Self {
            port1,
            _state: PhantomData,
        }
    }

    /// 手順1: Port Direction Register (PDR) の対象ビットを0にし、入力モードへ遷移
    #[inline(always)]
    fn into_input_mode(self) -> Port1Buttons<InputMode> {
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

impl Port1Buttons<InputMode> {
    /// 手順2: 入力モード設定済み かつ PWPRが `Unlocked` 状態のときのみ、内部プルアップ(PCR)を有効化可能
    #[inline(always)]
    fn enable_pullups(
        self,
        pfs: &pac::PFS,
        _pwpr_unlocked: &PwprGuard<'_, Unlocked>,
    ) -> Port1Buttons<PullUpReady> {
        pfs.p112pfs().modify(|_r, w| {
            w.pmr()
                .clear_bit()
                .asel()
                .clear_bit()
                .pdr()
                .clear_bit()
                .pcr()
                .set_bit()
        });
        pfs.p111pfs().modify(|_r, w| {
            w.pmr()
                .clear_bit()
                .asel()
                .clear_bit()
                .pdr()
                .clear_bit()
                .pcr()
                .set_bit()
        });
        pfs.p107pfs_ha()
            .modify(|_r, w| w.asel().clear_bit().pdr().clear_bit().pcr().set_bit());
        pfs.p106pfs_ha()
            .modify(|_r, w| w.asel().clear_bit().pdr().clear_bit().pcr().set_bit());
        Port1Buttons {
            port1: self.port1,
            _state: PhantomData,
        }
    }
}

impl Port1Buttons<PullUpReady> {
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

/// PFS/PMISCを含む周辺回路を消費し、設定後に生のレジスタへのアクセスを残さない。
///
/// 同じ周辺回路を使って複数の設定済みハンドルを作ることはできない。
/// ```compile_fail,E0382
/// use arduino_uno_r4_wifi_example00::configure;
/// fn duplicate(device: ra4m1::Peripherals) {
///     let first = configure(device);
///     let second = configure(device);
/// }
/// ```
pub fn configure(
    device: pac::Peripherals,
) -> (Port1Buttons<PullUpReady>, crate::led_matrix::LedMatrix) {
    let buttons = Port1Buttons::new(device.PORT1).into_input_mode();
    let mut protection = PfsProtection(device.PMISC);
    let unlocked = PwprGuard::new(&mut protection.0)
        .clear_b0wi()
        .enable_pfswe();
    let buttons = buttons.enable_pullups(&device.PFS, &unlocked);
    device
        .PFS
        .p003pfs()
        .modify(|_, w| w.pmr().clear_bit().pdr().clear_bit().pcr().clear_bit());
    device
        .PFS
        .p004pfs()
        .modify(|_, w| w.pmr().clear_bit().pdr().clear_bit().pcr().clear_bit());
    device
        .PFS
        .p011pfs()
        .modify(|_, w| w.pmr().clear_bit().pdr().clear_bit().pcr().clear_bit());
    device
        .PFS
        .p012pfs()
        .modify(|_, w| w.pmr().clear_bit().pdr().clear_bit().pcr().clear_bit());
    device
        .PFS
        .p013pfs()
        .modify(|_, w| w.pmr().clear_bit().pdr().clear_bit().pcr().clear_bit());
    device
        .PFS
        .p015pfs()
        .modify(|_, w| w.pmr().clear_bit().pdr().clear_bit().pcr().clear_bit());
    device
        .PFS
        .p204pfs()
        .modify(|_, w| w.pmr().clear_bit().pdr().clear_bit().pcr().clear_bit());
    device
        .PFS
        .p205pfs()
        .modify(|_, w| w.pmr().clear_bit().pdr().clear_bit().pcr().clear_bit());
    device
        .PFS
        .p206pfs()
        .modify(|_, w| w.pmr().clear_bit().pdr().clear_bit().pcr().clear_bit());
    device
        .PFS
        .p212pfs()
        .modify(|_, w| w.pmr().clear_bit().pdr().clear_bit().pcr().clear_bit());
    device
        .PFS
        .p213pfs()
        .modify(|_, w| w.pmr().clear_bit().pdr().clear_bit().pcr().clear_bit());
    device.PFS.p003pfs().modify(|_, w| w.asel().clear_bit());
    device.PFS.p004pfs().modify(|_, w| w.asel().clear_bit());
    device.PFS.p011pfs().modify(|_, w| w.asel().clear_bit());
    device.PFS.p012pfs().modify(|_, w| w.asel().clear_bit());
    device.PFS.p013pfs().modify(|_, w| w.asel().clear_bit());
    device.PFS.p015pfs().modify(|_, w| w.asel().clear_bit());
    device.PFS.p204pfs().modify(|_, w| w.asel().clear_bit());
    device.PFS.p205pfs().modify(|_, w| w.asel().clear_bit());
    device.PFS.p206pfs().modify(|_, w| w.asel().clear_bit());
    let _locked = unlocked.disable_pfswe().lock_b0wi();
    (
        buttons,
        crate::led_matrix::LedMatrix::new(device.PORT0, device.PORT2),
    )
}

// 正常なスコープ終了や早期returnでも保護を戻す。abort/電源断時のDropは保証しない。
struct PfsProtection(pac::PMISC);
impl Drop for PfsProtection {
    fn drop(&mut self) {
        self.0.pwpr.write(|w| w.b0wi().clear_bit());
        self.0.pwpr.write(|w| w.pfswe().clear_bit());
        self.0.pwpr.write(|w| w.b0wi().set_bit());
    }
}
