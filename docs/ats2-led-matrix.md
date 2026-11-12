# ATS2によるLEDマトリックスドライバの調査

調査日: 2026-11-09

## 結論と確認範囲

ATS2/Postiatsで、このリポジトリと同じ操作順序を型で制約する設計は可能と考えられる。
ATS2はCへ変換でき、GCやmalloc/freeを使わないArduinoプログラムの先行事例もある。
ただし、その事例は8ビットAVRの旧Arduino Unoであり、UNO R4 WiFi / RA4M1の実装実績を示すものではない。
今回確認した資料の範囲では、RA4M1用の完成済みATS2 LEDマトリックスドライバは確認できなかった。
[Arduino実装の論文](https://metasepi.org/doc/metasepi-icfp2015-arduino-ats.pdf)

ローカル環境のPATHには `patsopt`、`patscc`、`arm-none-eabi-gcc` がない。
この文書は調査・設計案であり、ATS2コードの型検査、C生成、実機向けリンク、点灯確認は未実施。
Rust版はボタン関連を削除し、`configure(device)` が `LedMatrix` のみを返す構成とした。

## 移植する現在の仕様

- 表示領域は12列×8行、フレームバッファは `u32` 3個の96ビット。
- ビット `x + y * 12` を座標 `(x, y)` に対応させる。
- PORT0のP003/P004/P011/P012/P013/P015、PORT2のP204/P205/P206/P212/P213を使用する。
- 各LEDは異なる2本のピンをLow/Highとして駆動し、他のLED用ピンは入力にする。
- 点灯ごとに入力化→出力ラッチのクリア→ペア駆動→保持→入力化の順序を守る。
- LED用マスク外のPDR/PODRは保持する。
- 初期化時はPWPRの保護解除、LED用PFS設定、PWPRの再保護を行う。
- ボタン入力やPORT1の設定は含めない。

Arduino公式実装でも96 LED、11本の配線、同じPORT0/PORT2マスクを確認できる。
ただし、公式APIと本リポジトリのビット順・High/Lowペアの順序が一致するとは限らない。
移植時は現在のRust版の96件の対応を基準に比較する。
[公式Arduino_LED_Matrix.h](https://github.com/arduino/ArduinoCore-renesas/blob/main/libraries/Arduino_LED_Matrix/src/Arduino_LED_Matrix.h)

## 型による設計案

ATS2の `absview` でハードウェア操作の権限を表し、状態を型引数に持たせる。
線形な証明を消費して次状態の証明を返すことで、同じ権限の二重利用を拒否できる。
証明は実行時には消去される。抽象viewの利用方法は言語の公式解説に示されている。
[Abstract Views and Viewtypes](https://ats-lang.github.io/FROZEN000/DOCUMENT/INT2PROGINATS/HTML/HTMLTOC/c4186.html)

| 状態 | 許可する操作 | 次状態 |
| --- | --- | --- |
| DirectionsCleared | LED用PODRをクリア | ReadyToDrive |
| ReadyToDrive | 有効な異なる2本のピンを駆動 | Active |
| Active | 所定の保持処理 | HoldCompleted |
| HoldCompleted | LED用PDRをクリア | DirectionsCleared |
| DirectionsCleared | 全LED消灯済みとして終了 | 権限を消費して終了 |

以下は `.sats` の宣言を想定した概念例であり、コンパイラでは未検証。
状態番号0/1/2/3は上表の順序に対応する。

```ats
absview matrix_v (addr, int)

fun clear_outputs {a:addr}
  (pf: matrix_v(a, 0) | ctx: ptr(a)): (matrix_v(a, 1) | void)

fun drive_pair
  {a:addr}{l,h:nat | l < 11; h < 11; l != h}
  (pf: matrix_v(a, 1) | ctx: ptr(a), low: int(l), high: int(h))
  : (matrix_v(a, 2) | void)

fun hold_uniform {a:addr}
  (pf: matrix_v(a, 2) | ctx: ptr(a)): (matrix_v(a, 3) | void)

fun turn_off {a:addr}
  (pf: matrix_v(a, 3) | ctx: ptr(a)): (matrix_v(a, 0) | void)
```

`ctx` は固定配置したドライバコンテキストへのポインタを想定する。
アドレスも証明の型引数に持たせ、操作権限とコンテキストを対応づける。
通常の `fun` の内部で実際のMMIO操作を行う。
レジスタ操作を伴う遷移を、実行時に消える `prfun` / `praxi` だけで表してはいけない。

依存型で追加できる制約は次の通り。

- ピクセル番号は `natLt(96)`、x座標は `natLt(12)`、y座標は `natLt(8)`。
- ピン番号は物理ビット番号ではなく11本の配線表に対する `natLt(11)` とする。
- Low/Highは静的条件 `l != h` を持たせる。
- 実行時に外部から得た番号は範囲検査し、検査成功の分岐で制約を得る。
- 任意のピンペアを利用者に公開せず、検証したピクセル→ピンペア変換をドライバ内部に隠す。

権限の複製だけでなく、生成も制限する必要がある。
`new(): (matrix_v(a, 0) | ptr(a))` 相当を何度でも呼べる設計ではシングルトンにならない。
起動処理で一度だけ作った線形のボード権限を、LEDドライバ初期化で消費する設計にする。
PWPRにも別の状態付き権限を設け、保護解除中の証明を持つ処理だけがPFSを設定できるようにする。

Rustの所有権は基本的にaffineで、値を使わずに破棄することができる。
ATS2の線形な権限では、通常終了する制御経路で明示的に次の操作へ渡す、または終了関数で消費する必要がある。
ただし、これも無限ループ、abort、電源断で必ず消灯するという保証にはならない。

## レジスタ操作との境界

ATS側に走査処理・ビットマップ・状態遷移を置き、小さなC層にvolatile MMIOアクセスを置く案が適している。
ATSとCは `ext#` / `mac#` などで接続できる。
[ATS2公式WikiのC interface](https://github.com/githwxi/ATS-Postiats/wiki/C-interface)

C層には以下を実装する。

- 正しいレジスタアドレス・アクセス幅によるPWPR/PFS/PDR/PODR/PCNTR1アクセス。
- LED用ピンだけを変更するマスク付き更新。
- 保持時間用のNOPまたはタイマ待機。
- 必要なら割り込みとの排他。volatileだけではread-modify-writeの競合を防げない。

C層とATSの宣言の一致は、型検査の前提として人手で確認する必要がある。
普通のATSポインタを読み書きするだけでは、MMIOのvolatileアクセスを保証できない。
公開APIから生ポインタや権限を作るキャストを隠し、C側や割り込み側にも同じピンを勝手に操作させない。
保持処理を呼んだことの証明と、実時間・電流・明るさの保証も区別する。

## ビルド方法

基本の流れは、ホスト上でATS2→C変換を行い、生成されたCをARM向けCコンパイラでコンパイルするもの。
ATS2コンパイラ自体をRA4M1上で動かす必要はない。
公式Arduinoコアのビルド設定もARM向けCコンパイルの参考になる。
[ATSコンパイラのオプション](https://ats-lang.sourceforge.net/RESOURCE/utils/atsoptman.html)、
[ArduinoCore-renesas/platform.txt](https://github.com/arduino/ArduinoCore-renesas/blob/main/platform.txt)

実装後に想定するコマンド例（現時点では参照先のATS/Cファイルは未作成）：

```sh
patsopt -tc -d matrix.dats
patsopt -o matrix_dats.c -d matrix.dats
arm-none-eabi-gcc -c matrix_dats.c -o matrix_dats.o \
  -mcpu=cortex-m4 -mthumb -mfpu=fpv4-sp-d16 -mfloat-abi=hard \
  -ffreestanding -ffunction-sections -fdata-sections \
  -I"$PATSHOME" -I"$PATSHOME/ccomp/runtime"
```

最後のコマンドはオブジェクト生成までで、完成したファームウェアを生成するレシピではない。
MMIO層のヘッダや、利用するATSライブラリに応じた追加設定も必要になる。

最初の移植では、既存のRust起動処理を残し、ATS生成オブジェクトをC ABI経由で呼ぶ方法を提案する。
Rust側の既存 `LedMatrix` とATS側が同じポートを同時に所有しないよう、初期化と操作権限を一方へ移す。
現在のターゲットが `thumbv7em-none-eabihf` なので、C側のABIもそれに合わせる。
Cargoへのオブジェクト組み込み設定は別途必要。

独立したATS2/Cファームウェアにする場合は、Cortex-Mのベクタテーブル、Reset_Handler、
.data/.bss初期化、RA4M1の初期設定、リンカスクリプトを用意する必要がある。
現在の `memory.x` は `cortex-m-rt` の `link.x` への追加設定なので、C用の完全なリンカスクリプトとしては使えない。
オプション設定領域（0x400）と書き込み方式も維持・確認する。

GCやヒープは使わず、固定長配列とスタック上の値を中心にする。
ATS2自体が「どの機能でもヒープ不要」というわけではない。
ランタイムのアロケータ設定を指定しない場合、確保処理は `atsruntime_*_undef` に接続される。
生成コードと未解決シンボルを調べ、malloc/free・例外・標準入出力などへの依存が混入していないことを確認する。
[ATS2ランタイムの実装](https://github.com/githwxi/ATS-Postiats/blob/master/ccomp/runtime/pats_ccomp_memalloc.h)

初期化が不要なモジュールには `ATS_DYNLOADFLAG 0` を検討できるが、実行時初期化が必要な値まで省略してはいけない。
[Dynamic Load](https://ats-lang.sourceforge.net/DOCUMENT/ATS2TUTORIAL/HTML/c191.html)

## 実装する場合の順序

1. `.sats` に状態遷移と番号制約を定義し、二重利用・消灯忘れ・同一ピン駆動・範囲外を拒否する型検査テストを作る。
2. ホスト用の疑似MMIOを使い、96ピクセルの対応と操作順序、対象外ピンの値の保持をRust版と比較する。
3. C ABI接続とヒープを使わないARMオブジェクトの生成を確認する。
4. RA4M1用MMIOへ置き換え、既存起動処理とリンクする。
5. 実機で全96ピクセルの配置、保持時間、消灯、ちらつきを確認する。

初期段階では、現在の三角数・ビット演算による配線計算をそのまま証明するより、
96件の対応表を使って範囲とピンの相違を検査する方が確認しやすい。
その後に計算式へ置き換える場合は、表との同値性を検証する。
