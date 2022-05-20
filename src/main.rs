#![no_std]
#![no_main]

const WDT_START_CODE: u32 = 0xcccc;
const WDT_ACCESSAVLE_CODE: u32 = 0x5555;
const WDT_REFRESH_CODE: u32 = 0xaaaa;

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
use cortex_m_rt::entry;
use cortex_m::delay;    // Delayを使う
use stm32f4::stm32f401;

#[entry]
fn main() -> ! {
    let dp = stm32f401::Peripherals::take().unwrap();   // デバイス用Peripheralsの取得
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();    // cortex-m Peripheralsの取得
    let mut delay = delay::Delay::new(cp.SYST, 84000000_u32);   // Delayの生成
    clock_init(&dp);    // クロック関連の初期化
    wdt_init(&dp);   // WDTの初期化
    loop {
        delay.delay_ms(4000_u32);           // delay
        wdt_refresh(&dp);
    }
}

fn clock_init(dp: &stm32f401::Peripherals) {

    // PLLSRC = HSI: 16MHz (default)
    dp.RCC.pllcfgr.modify(|_, w| w.pllp().div4());      // P=4
    dp.RCC.pllcfgr.modify(|_, w| unsafe { w.plln().bits(336) });    // N=336
    // PLLM = 16 (default)

    dp.RCC.cfgr.modify(|_, w| w.ppre1().div2());        // APB1 PSC = 1/2
    dp.RCC.cr.modify(|_, w| w.pllon().on());            // PLL On
    while dp.RCC.cr.read().pllrdy().is_not_ready() {    // 安定するまで待つ
        // PLLがロックするまで待つ (PLLRDY)
    }

    // データシートのテーブル15より
    dp.FLASH.acr.modify(|_,w| w.latency().bits(2));    // レイテンシの設定: 2ウェイト

    dp.RCC.cfgr.modify(|_,w| w.sw().pll());     // sysclk = PLL
    while !dp.RCC.cfgr.read().sws().is_pll() {  // SWS システムクロックソースがPLLになるまで待つ
    }
//  SYSCLK = 16MHz * 1/M * N * 1/P
//  SYSCLK = 16MHz * 1/16 * 336 * 1/4 = 84MHz
//  APB2 = 84MHz (SPI1 pclk)
}

fn wdt_init(dp: &stm32f401::Peripherals) {

    dp.IWDG.kr.write(|w| unsafe { w.bits(WDT_START_CODE) });    // Keyレジスタにスタートコードを書いて動作開始
    dp.IWDG.kr.write(|w| unsafe { w.bits(WDT_ACCESSAVLE_CODE) });   // アクセス保護の解除

    dp.IWDG.pr.modify(|_, w| unsafe { w.bits(3) });    // プリスケーラを 1/32 に設定
    dp.IWDG.rlr.modify(|_, w| unsafe { w.bits(4095) }); // リロード値を 4095 に設定

    while dp.IWDG.sr.read().rvu().bit() {
    }
    dp.IWDG.kr.write(|w| unsafe { w.bits(WDT_REFRESH_CODE) });  // keyレジスタにリフレッシュコードを書く
}

fn wdt_refresh(dp: &stm32f401::Peripherals) {
    dp.IWDG.kr.write(|w| unsafe { w.bits(WDT_REFRESH_CODE) });  // keyレジスタにリフレッシュコードを書く
}