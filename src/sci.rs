/*
use ra4m1::{self as pac, pfs::{p000pfs::P000PFS_SPEC, P000PFS_HA}, pmisc::{self, pwpr::PWPR_SPEC}, system::{mstpcra, MSTPCRA}};

/// すべてのUARTペリフェラルが実装する共通のインターフェース
pub trait Uart {
    /// 1バイトをブロッキング送信します。
    fn write_byte(&mut self, byte: u8);

    /// 1バイトをブロッキング受信します。
    fn read_byte(&mut self) -> u8;

    /// 文字列を送信します。
    /// ターミナルのために、自動的に `\n` を `\r\n` に変換します。
    fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
        }
    }
}

impl Uart for pac::SCI0 {
    fn read_byte(&mut self) -> u8 {
        while self.ssr().read().rdrf().bit_is_clear() {}
        self.rdr.read().rdr().bits()
    }

    fn write_byte(&mut self, byte: u8) {
         // 送信データレジスタが空になるまで待機 (TDREフラグ)
        while self.ssr().read().tdre().bit_is_clear() {}
        // データを送信データレジスタに書き込み
        self.tdr.write(|w| unsafe { w.tdr().bits(byte.into()) });
    }
}

impl Uart for pac::SCI1 {
    fn read_byte(&mut self) -> u8 {
        while self.ssr().read().rdrf().bit_is_clear() {}
        self.rdr.read().rdr().bits()
    }

    fn write_byte(&mut self, byte: u8) {
         // 送信データレジスタが空になるまで待機 (TDREフラグ)
        while self.ssr().read().tdre().bit_is_clear() {}
        // データを送信データレジスタに書き込み
        self.tdr.write(|w| unsafe { w.tdr().bits(byte.into()) });
    }
}


impl Uart for pac::SCI2 {
    fn read_byte(&mut self) -> u8 {
        while self.ssr().read().rdrf().bit_is_clear() {}
        self.rdr.read().rdr().bits()
    }
    fn write_byte(&mut self, byte: u8) {
         // 送信データレジスタが空になるまで待機 (TDREフラグ)
        while self.ssr().read().tdre().bit_is_clear() {}
        // データを送信データレジスタに書き込み
        self.tdr.write(|w| unsafe { w.tdr().bits(byte.into()) });
    }
}

impl Uart for pac::SCI9 {
    fn read_byte(&mut self) -> u8 {
        while self.ssr().read().rdrf().bit_is_clear() {}
        self.rdr.read().rdr().bits()
    }

    fn write_byte(&mut self, byte: u8) {
         // 送信データレジスタが空になるまで待機 (TDREフラグ)
        while self.ssr().read().tdre().bit_is_clear() {}
        // データを送信データレジスタに書き込み
        self.tdr.write(|w| unsafe { w.tdr().bits(byte.into()) });
    }

}


fn init_sci0(
     pfs: &pac::PFS,
)
{
    let dp = unsafe { pac::Peripherals::steal() };
    let system = &dp.SYSTEM;
    let pmisc = &dp.PMISC;

    pmisc.pwpr.write(|w|w.b0wi().clear_bit());
    pmisc.pwpr.write(|w|w.pfswe().set_bit()); // PFSレジスタの保護を解除


    pmisc.pwpr.write(|w|w.pfswe().clear_bit()); // PFSレジスタ保護
    pmisc.pwpr.write(|w|w.pfswe().set_bit());

    //system.mstpcra.modify(|_, w| w.mstpa22().clear_bit());
    //MSTPCRA::modify(&self, f);
    // pfs.p302pfs().write(|w|);
    //pac::pmisc::PWPR::
}
*/
