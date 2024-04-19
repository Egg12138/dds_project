//! re-write some command functions
use crate::data::*;
use crate::ddserror::DDSError;
use crate::log_func;
use crate::sys_clk_rate;
use crate::{config::*, data};
use colored::Colorize;
use config::{Config, File};
use lazy_static::lazy_static;
use serde_json;
use std::sync::RwLock;
use std::time::Duration;

//TODO: refactor definitions to another file.

const MSB: u8 = 0b0;
const LSB: u8 = 0b1;
/// Phase-Locked Loop, 输入与输出都是相位信息。可以用外部的参考信号控制
/// loop内部的震荡信号的频率和相位，使震荡信号与参考信号保持同步。
const PLL: u32 = 10;
const REF_CLK: u32 = 5000000;
const FTW_MASK: u32 = 0xffffffff;
// TODO: 应该把fmin, fmax全部作为const

const CSRAddr: u32 = 0x00;
const FR1Addr: u32 = 0x01;
const FR2Addr: u32 = 0x02;
const CFRAddr: u32 = 0x03;
const CFTW0Addr: u32 = 0x04;
const CPOW0Addr: u32 = 0x05;
const ACRAddr: u32 = 0x06;
const LSRRAddr: u32 = 0x07;
const RDWAddr: u32 = 0x08;
const FDWAddr: u32 = 0x09;

#[macro_export]
macro_rules! sys_clk_rate {
    () => {
        (REF_CLK * PLL)
    };
}

/// the maximum time interval o(At 500MSPS operation (SYC_CLK = 125MHz),
/// is : (1 / 125MHz) x 256= 2.048us
const MaxRampInterval: Duration = Duration::from_nanos(2048);
/// the minimum time interval o(At 500MSPS operation (SYC_CLK = 125MHz),
/// is : (1 / 125MHz) = 8.0ns
const MinRampInterval: Duration = Duration::from_nanos(8);
// BUG: overflow + too_small_clock_rate
// const MinFreqDeltaHz: u64 = sys_clk_rate!() as u64 >> 32;

lazy_static! {
    static ref AFP_SELECTOR: RwLock<AFPSelector> = RwLock::new(AFPSelector::NoModulation);
    #[deprecated]
    static ref AFP_SELECTORu8: RwLock<u8> = RwLock::new(0b0);
    static ref START_COMMANDS: RwLock<Vec<u64>> = RwLock::new(vec![]);
    static ref MEM_COMMANDS: RwLock<Vec<u64>> = RwLock::new(vec![]);
}

pub(self) enum Channel {
    Zero,
    One,
    Two,
    Three,
}
type Channels = (u8, u8, u8, u8);

/// convert the given literal (e.g: 0b0110) into a tuple of 4 channal on/off.
macro_rules! channels_frombits {
    ($v:expr) => {
        (($v & 0b0001), ($v & 0b0010), ($v & 0b0100), ($v & 0b1000))
    };
}

macro_rules! channel_id2bits {
    (0) => {
        0b0000
    };
    (1) => {
        0b0010
    };
    (2) => {
        0b0100
    };
    (3) => {
        0b1000
    };
    ($input:expr) => {
        0
    };
}

/// **Amplitude Frequency Phase Select(AFPSelector)** bits. `CFR[23:22]`
///  FallingDelta:
/// AFP_select = 0b01: frequency in Hz, max=clock x PLL, min = (clock x PLL)/ (2^32)
/// AFP_select = 0b10: amplitude:  max 1023, 10bit word
/// AFP_select = 0b11: phase: phase in degree, max=360, min = (360)/ (2^14)
///  RisindDelta:
// AFP_select = 0b01: frequency in Hz, max=clock x PLL, min = (clock x PLL)/ (2^32)
// AFP_select = 0b10: amplitude:  max 1023, 10bit word
// AFP_select = 0b11: phase: phase in degree, max=360, min = (360)/ (2^14)
#[derive(Clone)]
pub(self) enum AFPSelector {
    NoModulation,    // Linear Sweep Enable = X (LSE = X)
    AmpModulation,   // afp = 0b01, LSE = 0
    FreqModulation,  // afp = 0b10, LSE = 0
    PhaseModulation, // afp = 0b11, LSE = 0
}

macro_rules! AFP_2bits {
    ($a:expr) => {
        match $a {
            AFPSelector::NoModulation => 0b00,
            AFPSelector::AmpModulation => 0b01,
            AFPSelector::FreqModulation => 0b10,
            AFPSelector::PhaseModulation => 0b11,
        }
    };
}

macro_rules! onoff {
    ($s:expr) => {
        if $s {
            0b1
        } else {
            0b0
        }
    };
}

/// Modulatio Level bits `FR1[9:8]`
pub(self) enum ModulationLevel {
    /// 0b00
    Two,
    /// 0b01
    Four,
    /// 0b10
    Eight,
    /// 0b11
    Sixteen,
}

macro_rules! Modlv_2bits {
    ($m:expr) => {
        match $m {
            ModulationLevel::Two => 0b00,
            ModulationLevel::Four => 0b01,
            ModulationLevel::Eight => 0b10,
            ModulationLevel::Sixteen => 0b11,
        }
    };
}

/// RU/RD Profile Pin Assignments
pub(self) enum RURD {
    /// 0b00, RU/RD disabled
    Disabled,
    /// 0b01, Only Profile pin P2, P3
    Pin2Pin3,
    /// 0b10, Only Profile pin P3
    Pin3,
    /// 0b11, only SDIO_1,2,3 pins avaiable for RU/RD operation
    /// **forces the serial I/O** to be used **only** in **1-bit** mode,
    SDIOs,
}

macro_rules! RURD_2bits {
    ($r:expr) => {
        match $r {
            RURD::Disabled => 0b00,
            RURD::Pin2Pin3 => 0b01,
            RURD::Pin3 => 0b10,
            RURD::SDIOs => 0b11,
        }
    };
}

/// and actually, the Lock is unneccasry
pub(self) fn set_afp_select(to: AFPSelector) {
    if let Ok(mut afp_select) = AFP_SELECTOR.write() {
        *afp_select = to;
    } else {
        println!("blocked. AFPSelector is been read now.");
    }
}

macro_rules! phase2POW {
    ($p:expr) => {
        (phase / 360) << 14
    };
}

macro_rules! freq2FTW0 {
    ($fout:expr) => {
        ((($fout as u64) << 32) / sys_clk_rate!() as u64) as u64
    };
}

/// u16
/// convert the expected phase to the POW0 register value.
///  
macro_rules! ph2CPOW0 {
    ($phout:expr) => {
        ((0x3fff + 1) * $phout / 360) as u32
    };
}

/// ```rust
/// assert_eq!(channel_id2bits!(0), 0b1000);
/// assert_eq!(channel_id2bits!(1), 0b0100);
/// assert_eq!(channel_id2bits!(2), 0b0010);
/// assert_eq!(channel_id2bits!(3), 0b0001);
/// ```
/// return the channel btis when **only ONE** channel is enable.
#[deprecated(
    since = "0.1.5",
    note = "use `channel_bits2id` instead, 
which is implemented by simple pattern match"
)]
macro_rules! channel_id2bits {
    ($id:expr) => {{
        let mut output = 0b0001;
        for _ in 0..$id {
            output <<= 1;
        }
        output
    }};
}

#[cfg(debug_assertions)]
macro_rules! template_noparas_cmd {
    ($c:ident) => {
        let $c = str2cmd(stringify!($c)).unwrap();
        let Ok(_) = send_datapkg(quick_cmd2data($c)) else {
            log_func!(on_red: "failed to send datapkg");
            return;
        };
        log_func!();
    };
}

#[cfg(test)]
macro_rules! template_noparas_cmd {
    ($c:ident) => {
        assert!(1 + 1 == 2);
    };
}

macro_rules! template_genfn {
    ($name:ident) => {
        #[allow(unused)]
        fn $name() {
            template_noparas_cmd!($name);
        }
    };
}

macro_rules! template_genfns {
    ([$($name:ident),*]) => {
        $(
            template_genfn!{$name}
        )*
    };
}

template_genfns!([
    poweroff_dds,
    reset_dds,
    scan_dds,
    report_dds,
    update_dds,
    sync_dds,
    list_reset_dds,
    list_mode_dds,
    init_dds
]);

// with-paras cmds
// TODO: important, refactor from `Input` driven into the common DataPacket driven.
#[cfg(test)]
pub fn setinput_dds() -> Result<(), DDSError> {
    let builder = Config::builder()
        .add_source(File::with_name(LOCAL_CFG_PATH))
        .build();
    match builder {
        Ok(paras) => match paras.try_deserialize::<DDSInput>() {
            Ok(input) => {
                let datapkg = DataPacket::from(input);
                log_func!(on_bright_cyan:"41");
                send_datapkg(datapkg)
            }
            Err(e) => {
                log_func!(on_bright_cyan:"46");
                Err(e.into())
            }
        },
        Err(e) => {
            log_func!(on_red:"failed to build config");
            Err(e.into())
        }
    }
}

pub(crate) fn spi(
    // reg_id: u8,
    data: u64,
) {
    // TODO: how and what to do?
    // add the register address
    // match reg_id: ... u24, u16, u32, ...
    // let paras = (reg_id as u64) << 24 | data;
    let Ok(_) = quick_send_withparas(CommandTypes::SPI, data) else {
        log_func!(on_red:"Failed to send via direct SPI");
        return;
    };
}
pub(crate) fn list_length(len: u32) -> Result<(), DDSError> {
    // TODO refactor CommandTypes::ListLENGTH
    // NOTICE: 先不做这个！
    quick_send_withparas(CommandTypes::ListLength(len), len as u64)
}

enum SerialMode {
    SingleBit2Wire,
    SingleBit3Wire,
    TwoBits,
    FourBits,
}

macro_rules! serial_mode2bits {
    ($mode:expr) => {
        match $mode {
            SerialMode::SingleBit2Wire => 0b00,
            SerialMode::SingleBit3Wire => 0b01,
            SerialMode::TwoBits => 0b10,
            SerialMode::FourBits => 0b11,
        } as u16
    };
}

/// The content to be written to CSregister, requiring an extra bit for the register address.
/// Hence, the return is `u16`.
/// 7:4, enable bits
/// 3, must be 0
/// 2:1, serial I/O mode selct
pub fn CSR(channels: Channels) -> u16 {
    let csr = (CSRAddr as u16) << 8;
    let (ch0, ch1, ch2, ch3) = channels;
    let open = 0b0 << 3;
    let serial_io_mode = serial_mode2bits!(SerialMode::SingleBit2Wire) << 1;
    // let singlebit_3wire = serial_mode2bits!(SerialMode::SingleBit3Wire) << 1;
    // let serial_2bit = serial_mode2bits!(SerialMode::TwoBits) << 1;
    // let serial_3bit = serial_mode2bits!(SerialMode::ThreeBits) << 1;
    let order = MSB as u16;
    (csr | ch0 as u16 | ch1 as u16 | ch2 as u16 | ch3 as u16 | serial_io_mode | order | open) as u16
}

/// pump current mode enums
enum PumpC {
    Default75uA,
    Cur100uA,
    Cur125uA,
    Cur150uA,
}

macro_rules! pump2bits {
    ($p:expr) => {
        match $p {
            PumpC::Default75uA => 0b00,
            PumpC::Cur100uA => 0b01,
            PumpC::Cur125uA => 0b10,
            PumpC::Cur150uA => 0b11,
        } as u32
    };
}
/// DON'T NEED u32 cast only for `fr1` id bits.max: 24bits
pub fn FR1(pll_div: u8, mod_level: ModulationLevel) -> u32 {
    let fr1 = FR1Addr << 24; //Function Register 1 (FR1)—Address 0x01
    let vco_gain = 0b1_u32 << 23; //0 = the low range (system clock below 160 MHz) (default).
                                  //1 = the high range (system clock above 255 MHz).
    let pll_div_c = (pll_div as u32) << 18; //If the value is 4 or 20 (decimal) or between 4 and 20, the PLL is enabled and the value sets the
                                            //multiplication factor. If the value is outside of 4 and 20 (decimal), the PLL is disabled.
                                            // let pump_75uA = pump2bits!(PumpC::Default75uA) << 16; //00 (default) = the charge pump current is 75 μA
                                            // let pump_100uA = pump2bits!(PumpC::Cur100uA) << 16; //01 (default) = the charge pump current is 100 μA
                                            // let pump_125uA = pump2bits!(PumpC::Cur125uA) << 16; //10 (default) = the charge pump current is 125 μA
    let pump = pump2bits!(PumpC::Cur150uA) << 16; //11 (default) = the charge pump current is 150 μA
    let open1 = 0b0 << 15; //open
    let ppc_conf = 0b000 << 12; //The profile pin configuration bits control the configuration of the data and SDIO_x pins for the
                                //different modulation modes.
    let ru_rd = 0b00 << 10; //The RU/RD bits control the amplitude ramp-up/ramp-down time of a channel.
    let mod_lvl: u32 = (Modlv_2bits!(mod_level) as u32 & 0b00) << 8; //00 = 2-level modulation
                                                                     //01 = 4-level modulation
                                                                     //10 = 8-level modulation
                                                                     //11 = 16-level modulation
    let ref_clock = 0b0 << 7; //0 = the clock input circuitry is enabled for operation (default).
                              //1 = the clock input circuitry is disabled and is in a low power dissipation state.
    let pow_mode = 0b0 << 6; //0 = the external power-down mode is in fast recovery power-down mode (default). In this mode,
                             //when the PWR_DWN_CTL input pin is high, the digital logic and the DAC digital logic are
                             //powered down. The DAC bias circuitry, PLL, oscillator, and clock input circuitry are not powered down.
                             //1 = the external power-down mode is in full power-down mode. In this mode, when the
                             //PWR_DWN_CTL input pin is high, all functions are powered down. This includes the DAC and PLL,
                             //which take a significant amount of time to power up
    let sync_clock = 0b0 << 5; //0 = the SYNC_CLK pin is active (default).
                               //1 = the SYNC_CLK pin assumes a static Logic 0 state (disabled). In this state, the pin drive logic is
                               //shut down. However, the synchronization circuitry remains active internally to maintain normal
                               //device operation.
    let dac_ref = 0b0 << 4; //0 = DAC reference is enabled (default).
                            //1 = DAC reference is powered down
    let open2 = 0b00 << 2; //open
    let man_hard_sync = 0b0 << 1; //0 = the manual hardware synchronization feature of multiple devices is inactive (default).
                                  //1 = the manual hardware synchronization feature of multiple devices is active
    let man_soft_sync = 0b0; //0 = the manual software synchronization feature of multiple devices is inactive (default).
                             //1 = the manual software synchronization feature of multiple devices is active

    //composition of the command.
    (fr1 | vco_gain
        | pll_div_c
        | pump
        | open1
        | ru_rd
        | ppc_conf
        | pow_mode
        | mod_lvl
        | ref_clock
        | sync_clock
        | open2
        | dac_ref
        | man_hard_sync
        | man_soft_sync) as u32
}
pub fn FR2() -> u32 {
    let fr2 = (FR2Addr as u32) << 16;

    // set register map
    // for all channels:
    // ---
    let auto_clr_sweep_acc = 0b0 << 15;
    let clr_sweep_acc = 0b0 << 14;
    let auto_clr_ph_acc = 0b0 << 13;
    let clear_phase_acc = 0b0 << 12;
    // ---

    let open1 = 0b0000 << 8;
    let auto_sync_enable = 0b0 << 7;
    let multi_dev_sync_master_enable = 0b0 << 6;
    let multi_dev_sync_status = 0b0 << 5;
    let multi_dev_sync_mask = 0b0 << 5;
    let open2 = 0b00 << 2;
    let sys_clock_off = 0b0;

    (fr2 | clear_phase_acc
        | auto_clr_ph_acc
        | clr_sweep_acc
        | auto_clr_sweep_acc
        | open1
        | auto_sync_enable
        | multi_dev_sync_status
        | multi_dev_sync_status
        | multi_dev_sync_master_enable
        | multi_dev_sync_mask
        | open2
        | sys_clock_off) as u32
}

// TODO: decide the input type should be bool type, customized type or u8/u16/u32?
/// NOTICE: 每个channel都有同样的这个progile registers设置，
///The slope of the linear sweep is set by the intermediate step size
/// (delta-tuning word) between S0 (memory 0 or actual value) and E0 (memory 1 see CW_register) and the time spent
/// (sweep ramp rate word) at each step. The resolution of the
/// delta-tuning word is 32 bits for frequency, 14 bits for phase, and
/// 10 bits for amplitude. The resolution for the delta ramp rate
/// word is eight bits
pub fn CFR(
    afp: AFPSelector,
    lsweep_nodwell: bool,  // default = 0 (inactive)
    lsweep_enable: bool,   // default = 0 (inactive)
    srr_at_ioupdate: bool, // default = 0: linear sweep ramp rate timer is loaded only upon timeout
) -> u32 {
    // NOTICE: u24寄存器不会受到OF影响，但是我们需要保证统一数据格式。
    let cfr = (CFRAddr as u32) << 24;
    let AFP_select = AFP_2bits!(afp) << 22;
    let open1 = 0b0 << 16;
    let lsweep_nodwell = onoff!(lsweep_nodwell) << 15;
    let lsweep_enable = onoff!(lsweep_enable) << 14;
    let load_srr_at_ioupdate = onoff!(srr_at_ioupdate) << 13;
    let open2 = 0 << 11;
    let mustbe0 = 0 << 10;
    let dac_full_scale_current_ctr = 0b11 << 8; // CFR[15:8]default = 0x03
    let dig_power_down = 0b0 << 7;
    let dac_power_down = 0b0 << 6;
    let pipe_delay = 0b0 << 5;
    let auto_clr_sweep = 0b0 << 4;
    let clr_sweep = 0b0 << 3;
    let auto_clr_ph = 0b0 << 2;
    let clr_ph_acc = 0b1 << 1;
    let sin_out = 0b0;
    (cfr | AFP_select
        | open1
        | lsweep_nodwell
        | lsweep_enable
        | load_srr_at_ioupdate
        | open2
        | mustbe0
        | dac_full_scale_current_ctr
        | dig_power_down
        | dac_power_down
        | pipe_delay) as u32
}

// TODO: OOP, self.pll, self.clock
// TODO: frequency input float? or u32? or f64?
/// **VCO**-related: PLL
/// each channel has a decicated 32-bit frequency tunning word
/// f_out = FTW * f_s / 2^32, f_S = REF_CLK_RATE
/// FTW = f_out * 2^32 / f_s
/// NOTICE: 每个channel都有同样的这个progile registers设置，
pub fn CFTW(freq: u32) -> u64 {
    let cftw = (CFTW0Addr as u64) << 32;
    // LEARN: core_clock
    // LEARN: how to get Frequency Tuning Word (FTW) from desired output freq
    let cftw_value = freq2FTW0!(freq);
    cftw | cftw_value
}

/// NOTICE: 每个channel都有同样的这个progile registers设置，
pub fn CPOW(phase: u32) -> u32 {
    let cpow = 0x05 << 16;
    let open = 0b00 << 14;
    let cpow_value = ph2CPOW0!(phase);
    cpow | open | cpow_value
}

/// 24 bits + 1
/// amplitude ramp rate[23:16] default: N/A
/// NOTICE: 每个channel都有同样的这个progile registers设置，
pub fn ACR(multiplier_enable: bool, amp: u32) -> u32 {
    let acr = 0x06 << 24;
    let amp_ramp_rate = 0x00 << 15;
    let step_size = 0b00 << 14;
    let open = 0b0 << 13;
    let multiplier_enable = onoff!(multiplier_enable) << 12 as u32;
    let ramp_enable = 0b0 << 11;
    let arr_atioupdate = 0b0 << 10;
    let amplitude = amp & 0x3ff;
    acr | amp_ramp_rate
        | step_size
        | open
        | multiplier_enable
        | ramp_enable
        | arr_atioupdate
        | amplitude
}

/// Linear Sweep Ramp Rate
pub fn LSRR(falling_sweep_ramp_rate: u32, rising_sweep_ramp_rate: u32) -> u16 {
    // let lsrr = 0x07 << 16;
    let sync_clk = sys_clk_rate!() / 4;
    let frr = if falling_sweep_ramp_rate > (0xff / sync_clk) {
        0xff / sync_clk
    } else if falling_sweep_ramp_rate < 1 / sync_clk {
        1 / sync_clk
    } else {
        panic!("Impossible! Falling Ramp Rate set incorrectly! ");
    } * sync_clk;
    let frr = frr as u16;
    // let frr = (falling_sweep_ramp_rate * sync_clk) as u32 << 8;
    let rrr = if rising_sweep_ramp_rate > (0xff / sync_clk) {
        0xff / sync_clk
    } else if rising_sweep_ramp_rate < 1 / sync_clk {
        sync_clk
    } else {
        panic!("Impossible! Rising Ramp Rate set incorrectly! ");
    } * sync_clk;
    let rrr = rrr as u16;
    // lsrr | frr | rrr
    frr | rrr
}

/// matchh afp selector:
/// case Disable
/// case Amp
/// case Freq
/// case Phase
///
/// in each case, macro check the validation of the input `size`,
/// if exceeded, warn else make the input `size` compatible for the 32bit register format
macro_rules! afp_match_WORD {
    ($w:expr) => {
        {
        if let Ok(afp) = AFP_SELECTOR.read() {
            match *afp {
                AFPSelector::NoModulation => {
                    log_func!("no modulation is selected");
                    0
                }
                AFPSelector::AmpModulation => {
                    if $w > 0x3ff {
                        // TODO Max might not be 1024
                        log_func!(red:"amplitude modulation is selected, max 1024");
                        (0x3ff << 22 )
                    } else {
                        (($w & 0x3ff) ) << 22
                    }
                }
                AFPSelector::FreqModulation => {
                    // BUG bitwise will overflow
                    if $w > sys_clk_rate!() {
                        println!("{} {} Hz", "frequency modulation selected, max ".red(), sys_clk_rate!(), );
                        (((0xffffffffu64 + 1) )& 0xffffffff  ) as u32
                    } else {
                        (((0xffffffffu64 + 1) * ($w as u64) / sys_clk_rate!() as u64) & 0xffffffff) as u32
                    }
                }
                AFPSelector::PhaseModulation => ((((0x3fff + 1) * $w / 360) & 0x3fff) << 18) as u32
            }
        } else {
            log_func!(on_red:"RwLock<AFPSelector> is not accessed");
            0
        }
    }
    };
}

// TODO: 匹配0b01, 0b10顺序有错误!!!
/// LSR Rising Delta Word
/// NOTICE: 每个channel都有同样的这个progile registers设置，
pub fn RDW(step: u32) -> u32 {
    // let rdw = 0x08 << 32;
    let rdw_value = afp_match_WORD!(step);
    rdw_value
}

/// NOTICE: 每个channel都有同样的这个progile registers设置，
pub fn FDW(step: u32) -> u32 {
    // let fdw = 0x09 << 32;
    let fdw_value = afp_match_WORD!(step);
    fdw_value
}

pub fn CW(cwid: u8, word: u32) -> u32 {
    // The range start..end contains all values with start <= x < end. It is empty if start >= end.
    assert!(
        (1..16).contains(&cwid),
        "select a memory slot between 1 and 15"
    );
    // cw1=>0x0A, 0w15=>0x18
    // let cw_n = ((cwid + 0x09) as u32) << 32;
    // TODO : is it possible to convert core_clock as static variable?
    let core_clock = sys_clk_rate!();
    let cw_n_value = afp_match_WORD!(word);
    cw_n_value
}

macro_rules! send_via_spi {
    ($($c:ident),+$(,)?) => {
        $(
            spi($c as u64);
        )*
    };
}

pub fn init_viaSPI(pll_div: u8, afp: AFPSelector, mod_lvl: ModulationLevel, send: bool) {
    let csr_spi = (CSR((1, 1, 1, 1)) as u64) << 40;
    let fr1_spi = FR1(pll_div, ModulationLevel::Two) as u64;
    let fr2_spi = FR2();
    let cfr_spi = CFR(afp, false, false, false);
    if send {
        send_via_spi!(csr_spi, fr1_spi, fr2_spi, cfr_spi);
    }
}

pub fn set_frequency(channel: u8, freq: u32, send: bool) -> u64 {
    let csr_spi: u64 = (CSR(channels_frombits!(channel)) as u64) << 40;
    let cftw_spi = CFTW(freq) as u64;
    let freq_cmd = (csr_spi | cftw_spi) as u64;
    if send {
        spi(freq_cmd);
    }
    freq_cmd
}

pub fn set_amplitude(channel: u8, amp: u32, send: bool) -> u64 {
    let csr_spi: u64 = (CSR(channels_frombits!(channel)) as u64) << 32;
    if amp > 1024 {
        log_func!(red:"max amplitude 1023, clamped");
    }
    let acr_spi = ACR(true, amp) as u64;
    let amp_cmd = (csr_spi | acr_spi) as u64;
    if send {
        spi(amp_cmd);
    }
    amp_cmd
}

/// channel: is a `u8` with 4 high bits are 0, indicating the status of each channel
/// for example, channel = `0b1101` => channel 0,1,3 are enable.
/// phase: phase in degree, max = 360, min = 360 / 2^14
pub fn set_phase(channel: u8, phase: u32, send: bool) -> u64 {
    let csr_spi: u64 = (CSR(channels_frombits!(channel)) as u64) << 24;
    if phase > 360 {
        log_func!(red:"max phase 360, clamped");
    }
    let cph_spi = CPOW(phase) as u64;
    let phase_cmd = (csr_spi | cph_spi) as u64;
    if send {
        spi(phase_cmd);
    }
    phase_cmd
}

/// change between levels is made via **profile pins** (chX->PinX)
/// 2-level modulation is the default mode
/// ch2B2mod: channels to be the 2-level modulation
pub fn set_2mod_freq(chs2B2set: u8, freq: u32, send: bool) -> (u64, u64, u64) {
    let csr_spi = CSR(channels_frombits!(chs2B2set)) as u64;
    set_afp_select(AFPSelector::FreqModulation);
    let afp = AFPSelector::FreqModulation;
    let cfr_spi = CFR(afp, false, false, false) as u64;
    let cw_spi = CW(1, freq) as u64;
    if send {
        send_via_spi!(csr_spi, cfr_spi, cw_spi,);
    }
    (csr_spi, cfr_spi, cw_spi)
}

pub fn set_2mod_amp(chs2B2set: u8, amp: u32, send: bool) -> (u64, u64, u64) {
    let csr_spi = CSR(channels_frombits!(chs2B2set)) as u64;
    set_afp_select(AFPSelector::AmpModulation);
    let afp = AFPSelector::AmpModulation;
    let cfr_spi = CFR(afp, false, false, false) as u64;
    let cw_spi = CW(1, amp) as u64;
    if send {
        send_via_spi!(csr_spi, cfr_spi, cw_spi);
    }
    (csr_spi, cfr_spi, cw_spi)
}
pub fn set_2mod_phase(chs2B2set: u8, phase: u32, send: bool) -> (u64, u64, u64) {
    let csr_spi = CSR(channels_frombits!(chs2B2set)) as u64;
    set_afp_select(AFPSelector::PhaseModulation);
    let afp = AFPSelector::PhaseModulation;
    let cfr_spi = CFR(afp, false, false, false) as u64;
    let cw_spi = CW(1, phase) as u64;
    if send {
        send_via_spi!(csr_spi, cfr_spi, cw_spi);
    }
    (csr_spi, cfr_spi, cw_spi)
}

macro_rules! tou8 {
    ($x:expr) => {
        ($x & 0xff) as u8
    };
}
macro_rules! tou16 {
    ($x:expr) => {
        ($x & 0xffff) as u16
    };
}

/// channel: channel(s) to be changed
/// r_time: ramp time **in milliseconds**
/// f_init: start frequency of the ramp, **in Hz**, max = clock x PLL, min = (clock x PLL) / 2^32
/// f_final: end frequency of the ramp, **in Hz**, max = clock x PLL, min = (clock x PLL) / 2^32
pub fn ramp_freq(
    channel: u8,
    time_step: Duration,
    f_init: u32,
    f_final: u32,
    send: bool,
) -> Result<(u64, u64, u64, u64, u64, u64, u64), DDSError> {
    let clk = sys_clk_rate!();

    if f_init > f_final {
        log_func!(on_red:"init frequency should be less than final frequency");
        return Err(DDSError::IllegalArgument);
    }

    if f_init > clk || f_final > clk {
        eprintln!(
            "{}{}",
            "Max frequency is ".on_red(),
            clk.to_string().on_bright_green(),
        );
        return Err(DDSError::IllegalArgument);
    }

    let frange = f_final - f_init;
    let delta_f_Min_hz = (sys_clk_rate!() as f64 / 2.0f64.powf(32.0));

    // the samples nums of **minimum** ramp time interval
    let n_samples_tmin = time_step.div_duration_f64(MinRampInterval) as u64;

    // the samples nums of **minimun** frequncy delta(min steplen, most steps)
    let n_samples_fmin = (frange as f64 / delta_f_Min_hz) as u64;

    // RR interval starts with `r_time / n_samples_fmin (the most steps case)`
    let mut delta_time = time_step.as_nanos() / (n_samples_fmin as u128);
    let mut delta_f = frange as f64 / n_samples_tmin as f64;

    if (frange as f64) < delta_f_Min_hz {
        eprintln!(
            "{}{}",
            "Ramp step too short, min step is".on_red(),
            delta_f_Min_hz.to_string().on_bright_green()
        );
        return Err(DDSError::IllegalArgument);
    }

    if delta_time > MaxRampInterval.as_nanos() {
        eprintln!(
            "{}{:?}",
            "Ramp time is too long, max time step is ".on_red(),
            MaxRampInterval
        );
        return Err(DDSError::IllegalArgument);
    }

    if time_step < MinRampInterval {
        eprintln!(
            "{}{:?}",
            "Ramp time is too short, min time step is ".on_red(),
            MinRampInterval
        );
        return Err(DDSError::IllegalArgument);
    }

    let n_samples = if delta_f < delta_f_Min_hz && delta_time > MinRampInterval.as_nanos() {
        delta_f = delta_f_Min_hz;
        let n_samples = n_samples_fmin;
        n_samples
        // FIXME: is here really `&&` ?
    } else if delta_f > delta_f_Min_hz && delta_time < MinRampInterval.as_nanos() {
        delta_time = MinRampInterval.as_nanos();
        let n_samples = n_samples_tmin;
        n_samples
    } else {
        log_func!(purple:"Shouldn't reached here!");
        n_samples_fmin.min(n_samples_tmin)
    };

    let delta_time = delta_time as u64; // re-cast.
    let delta_time_dur = Duration::from_nanos(delta_time);
    println!("ramp time rate (time interval) is {:?}", delta_time_dur);
    println!("ramp value rate (freq delta) is {}", delta_f);
    println!("number of samples: {}", n_samples);
    println!(
        " total time: {}, total change {}",
        n_samples * delta_time,
        n_samples as f64 * delta_f
    );

    set_afp_select(AFPSelector::FreqModulation);
    let delta_time = tou16!(delta_time);
    let csr_spi = CSR(channels_frombits!(channel)) as u64;
    let lsrr_spi = LSRR(delta_time as u32, delta_time as u32) as u64;
    // BUG: delta_f 精度到底多少,现在这样转换肯定全0
    let fdw_spi = FDW(delta_f as u32) as u64;
    let rdw_spi = RDW(delta_f as u32) as u64;
    let cfr_spi = CFR(AFPSelector::FreqModulation, false, true, false) as u64;
    let cftw_spi = CFTW(f_init);
    let cw_spi = CW(1, f_final) as u64;

    if send {
        send_via_spi!(csr_spi, lsrr_spi, fdw_spi, rdw_spi, cfr_spi, cftw_spi, cw_spi);
    }
    log_func!();
    Ok((
        csr_spi, lsrr_spi, fdw_spi, rdw_spi, cfr_spi, cftw_spi, cw_spi,
    ))
}

// NOTICE: 阅读具体DAC参数
/// it matters that the amplitude is with the unit `percent`.
/// **ratio of DAC's full-scale current**,
pub fn ramp_amp(
    channel: u8,
    time_step: Duration,
    a_init: u32,
    a_final: u32,
    send: bool,
) -> Result<(u64, u64, u64, u64, u64, u64, u64), DDSError> {
    let clk = sys_clk_rate!();

    if a_init > a_final {
        log_func!(on_red:"init amplitude should be less than final frequency");
        return Err(DDSError::IllegalArgument);
    }

    if a_init > clk || a_final > clk {
        log_func!(on_red:
            "Max Amplitude is 1023"
        );
        return Err(DDSError::IllegalArgument);
    }
    let arange = a_final - a_init;
    let delta_a_Min_dac = 1.0;

    // the samples nums of **minimum** ramp time interval
    let n_samples_tmin = time_step.div_duration_f64(MinRampInterval) as u64;

    // the samples nums of **minimun** frequncy delta(min steplen, most steps)
    let n_samples_amin = (arange as f64 / delta_a_Min_dac) as u64;

    // RR interval starts with `r_time / n_samples_fmin (the most steps case)`
    let mut delta_time = time_step.as_nanos() / (n_samples_amin as u128);
    let mut delta_a = arange as f64 / n_samples_tmin as f64;

    if (arange as f64) < delta_a_Min_dac {
        eprintln!(
            "{}{}",
            "Ramp step too short, min step is".on_red(),
            delta_a_Min_dac.to_string().on_bright_green()
        );
        return Err(DDSError::IllegalArgument);
    }

    if delta_time > MaxRampInterval.as_nanos() {
        eprintln!(
            "{}{:?}",
            "Ramp time is too long, max time step is ".on_red(),
            MaxRampInterval
        );
        return Err(DDSError::IllegalArgument);
    }

    if time_step < MinRampInterval {
        eprintln!(
            "{}{:?}",
            "Ramp time is too short, min time step is ".on_red(),
            MinRampInterval
        );
        return Err(DDSError::IllegalArgument);
    }

    let n_samples = if delta_a < delta_a_Min_dac && delta_time > MinRampInterval.as_nanos() {
        delta_a = delta_a_Min_dac;
        let n_samples = n_samples_amin;
        n_samples
        // FIXME: is here really `&&` ?
    } else if delta_a > delta_a_Min_dac && delta_time < MinRampInterval.as_nanos() {
        delta_time = MinRampInterval.as_nanos();
        let n_samples = n_samples_tmin;
        n_samples
    } else {
        log_func!(purple:"Shouldn't reached here!");
        n_samples_amin.min(n_samples_tmin)
    };

    let delta_time = delta_time as u64; // re-cast.
    let delta_time_dur = Duration::from_nanos(delta_time);
    println!("ramp time rate (time interval) is {:?}", delta_time_dur);
    println!(
        "ramp value rate (amplitude delta) is {} of DAC full-scale",
        delta_a
    );
    println!("number of samples: {}", n_samples);
    println!(
        " total time: {}, total change {}",
        n_samples * delta_time,
        n_samples as f64 * delta_a
    );

    set_afp_select(AFPSelector::AmpModulation);
    let delta_time = tou16!(delta_time);
    let csr_spi = CSR(channels_frombits!(channel)) as u64;
    let lsrr_spi = LSRR(delta_time as u32, delta_time as u32) as u64;
    let fdw_spi = FDW(delta_a as u32) as u64;
    let rdw_spi = RDW(delta_a as u32) as u64;
    let cfr_spi = CFR(AFPSelector::AmpModulation, false, true, false) as u64;
    let cftw_spi = CFTW(a_init);
    let cw_spi = CW(1, a_final) as u64;

    if send {
        send_via_spi!(csr_spi, lsrr_spi, fdw_spi, rdw_spi, cfr_spi, cftw_spi, cw_spi);
    }
    log_func!();
    Ok((
        csr_spi, lsrr_spi, fdw_spi, rdw_spi, cfr_spi, cftw_spi, cw_spi,
    ))
}

// NOTICE: 阅读具体DAC参数
/// it matters that the amplitude is with the unit `percent`.
/// **ratio of DAC's full-scale current**,
pub fn ramp_phase(
    channel: u8,
    time_step: Duration,
    // TODO improve the solution, use f32 instead.
    p_initf: f64,
    p_finalf: f64,
    send: bool,
) -> Result<(u64, u64, u64, u64, u64, u64, u64), DDSError> {
    let p_init = p_initf as u32;
    let p_final = p_finalf as u32;
    let clk = sys_clk_rate!();

    if p_initf > p_finalf {
        log_func!(on_red:"init frequency should be less than final frequency");
        return Err(DDSError::IllegalArgument);
    }

    if p_initf > 360.0 || p_finalf > 360.0 {
        log_func!(on_red:"MAX degree of phase is 360.0");
        return Err(DDSError::IllegalArgument);
    }
    let prangeu = p_final - p_init;
    let prange = p_finalf - p_initf;
    let delta_p_min = (360.0) / (2.0f64.powf(14.0));

    // the samples nums of **minimum** ramp time interval
    let n_samples_tmin = time_step.div_duration_f64(MinRampInterval) as u64;

    // the samples nums of **minimun** frequncy delta(min steplen, most steps)
    let n_samples_pmin = (prangeu as f64 / delta_p_min) as u64;

    // RR interval starts with `r_time / n_samples_fmin (the most steps case)`
    let mut delta_time = time_step.as_nanos() / (n_samples_pmin as u128);
    let mut delta_p = prange / n_samples_tmin as f64;

    if prange < delta_p_min {
        eprintln!(
            "{}{}",
            "Ramp step too short, min step is".on_red(),
            delta_p_min.to_string().on_bright_green()
        );
        return Err(DDSError::IllegalArgument);
    }

    if delta_time > MaxRampInterval.as_nanos() {
        eprintln!(
            "{}{:?}",
            "Ramp time is too long, max time step is ".on_red(),
            MaxRampInterval
        );
        return Err(DDSError::IllegalArgument);
    }

    if time_step < MinRampInterval {
        eprintln!(
            "{}{:?}",
            "Ramp time is too short, min time step is ".on_red(),
            MinRampInterval
        );
        return Err(DDSError::IllegalArgument);
    }

    let n_samples = if delta_p < delta_p_min && delta_time > MinRampInterval.as_nanos() {
        delta_p = delta_p_min;
        let n_samples = n_samples_pmin;
        n_samples
        // FIXME: is here really `&&` ?
    } else if delta_p > delta_p_min && delta_time < MinRampInterval.as_nanos() {
        delta_time = MinRampInterval.as_nanos();
        let n_samples = n_samples_tmin;
        n_samples
    } else {
        log_func!(purple:"Shouldn't reached here!");
        n_samples_pmin.min(n_samples_tmin)
    };

    let delta_time = delta_time as u64; // re-cast.
    let delta_time_dur = Duration::from_nanos(delta_time);
    println!("ramp time rate (time interval) is {:?}", delta_time_dur);
    println!(
        "ramp value rate (amplitude delta) is {} of DAC full-scale",
        delta_p
    );
    println!("number of samples: {}", n_samples);
    println!(
        " total time: {}, total change {}",
        n_samples * delta_time,
        n_samples as f64 * delta_p
    );

    set_afp_select(AFPSelector::PhaseModulation);
    let delta_time = tou16!(delta_time);
    let csr_spi = CSR(channels_frombits!(channel)) as u64;
    let lsrr_spi = LSRR(delta_time as u32, delta_time as u32) as u64;
    // BUG: solution is : using u32  causes 0
    let fdw_spi = FDW(delta_p as u32) as u64;
    let rdw_spi = RDW(delta_p as u32) as u64;
    let cfr_spi = CFR(AFPSelector::PhaseModulation, false, true, false) as u64;
    let cftw_spi = CFTW(p_init);
    let cw_spi = CW(1, p_final) as u64;

    if send {
        send_via_spi!(csr_spi, lsrr_spi, fdw_spi, rdw_spi, cfr_spi, cftw_spi, cw_spi);
    }
    log_func!();
    Ok((
        csr_spi, lsrr_spi, fdw_spi, rdw_spi, cfr_spi, cftw_spi, cw_spi,
    ))
}

pub fn begin_with_one(command: DataStream) {}

pub fn begin_with_command_serial() {
    quick_gen_binary();
}

#[test]
fn test_gen_fn_exits() {
    poweroff_dds();
    reset_dds();
    scan_dds();
    report_dds();
    update_dds();
    sync_dds();
    list_reset_dds();
    list_mode_dds();
    init_dds();
}

/// considering the u64 -> u32 conversion, the FTW value should be checked
#[test]
fn ctwf_value_right() {
    assert_eq!(freq2FTW0!(1000000000), 0x00000000_00000001);
    assert_eq!(CFTW(0x11210245), 0);
}
