//! re-write some command functions
use crate::data::*;
use crate::ddserror::DDSError;
use crate::log_func;
use crate::{config::*, data};
use colored::Colorize;
use config::{Config, File};
use lazy_static::lazy_static;
use serde_json;
use std::sync::RwLock;

const MSB: u8 = 0b0;
const LSB: u8 = 0b1;
/// Phase-Locked Loop, 输入与输出都是相位信息。可以用外部的参考信号控制
/// loop内部的震荡信号的频率和相位，使震荡信号与参考信号保持同步。
const PLL: u32 = 10;
const REF_CLK: u32 = 5000000;
const FTW_MASK: u32 = 0xffffffff;

lazy_static! {
    static ref AFP_SELECTOR: RwLock<AFPSelector> = RwLock::new(AFPSelector::NoModulation);
    #[deprecated]
    static ref AFP_SELECTORu8: RwLock<u8> = RwLock::new(0b0);
}

///  FallingDelta:
/// AFP_select = 0b01: frequency in Hz, max=clock x PLL, min = (clock x PLL)/ (2^32)
/// AFP_select = 0b10: amplitude:  max 1023, 10bit word
/// AFP_select = 0b11: phase: phase in degree, max=360, min = (360)/ (2^14)
///  RisindDelta:
// AFP_select = 0b01: frequency in Hz, max=clock x PLL, min = (clock x PLL)/ (2^32)
// AFP_select = 0b10: amplitude:  max 1023, 10bit word
// AFP_select = 0b11: phase: phase in degree, max=360, min = (360)/ (2^14)
pub(self) enum AFPSelector {
    NoModulation,
    Frequency, // afp = 0b10
    Amplitude,// afp = 0b01
    PhaseDegree, // afp = 0b11
}

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

macro_rules! sys_clk_rate {
    () => {
        (REF_CLK * PLL)
    };
}
macro_rules! freq2FTW0 {
    ($fout:expr) => {
        ((($fout as u64) << 32) / sys_clk_rate!() as u64) as u32
    };
}

/// u16
macro_rules! ph2CPOW0 {
    ($phout:expr) => {
        ((0x3fff + 1) * $phout / 360) as u16
    };
}

type Channel = (u8, u8, u8, u8);

/// convert the given literal (e.g: 0b0110) into a tuple of 4 channal on/off.
macro_rules! allch_frombits {
    ($v:literal) => {
        let ch0 = ($v & 0b1000);
        let ch1 = ($v & 0b0100);
        let ch2 = ($v & 0b0010);
        let ch3 = ($v & 0b0001);
        (ch0, ch1, ch2, ch3)
    };
}

macro_rules! channel_id2bits_faster {
    (0) => {
        0b1000
    };
    (1) => {
        0b0100
    };
    (2) => {
        0b0010
    };
    (3) => {
        0b0001
    };
    ($input:expr) => {
        0
    };
}

/// ```rust
/// assert_eq!(channel_id2bits!(0), 0b1000);
/// assert_eq!(channel_id2bits!(1), 0b0100);
/// assert_eq!(channel_id2bits!(2), 0b0010);
/// assert_eq!(channel_id2bits!(3), 0b0001);
/// ```
#[deprecated(
    since = "0.1.5",
    note = "use `channel_bits2id` instead, 
which is implemented by simple pattern match"
)]
macro_rules! channel_id2bits {
    ($id:expr) => {
        let mut output = 0b1000;
        for _ in 0..$input {
            output >>= 1;
        }
        output
    };
}

macro_rules! channel {
    ($id:literal) => {
        let bits: u8 = 0b0;
        // IMPL: bitwise
    };
}

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
pub fn setinput_dds() -> Result<(), DDSError> {
    let builder = Config::builder()
        .add_source(File::with_name(LOCAL_CFG_PATH))
        .build();
    match builder {
        Ok(paras) => match paras.try_deserialize::<Input>() {
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

pub fn CSR(channel: Channel) -> u8 {
    let csr = 0x20 << 8;
    let (ch0, ch1, ch2, ch3) = channel;
    let open = 0b0 << 3;
    let singlebit_2wire = 0b00 << 1;
    let singlebit_3wire = 0b01 << 1;
    let serial_2bit = 0b10 << 1;
    let serial_3bit = 0b11 << 1;

    csr | ch0 | ch1 | ch2 | ch3 | singlebit_2wire | MSB | open
}

pub fn FR1(pll_div: u8, mod_level: u8) -> u32 {
    let fr1 = 0x01 << 24; //Function Register 1 (FR1)—Address 0x01
    let vco_gain = 0b1 << 23; //0 = the low range (system clock below 160 MHz) (default).
                              //1 = the high range (system clock above 255 MHz).
    let pll_div_c = pll_div << 18; //If the value is 4 or 20 (decimal) or between 4 and 20, the PLL is enabled and the value sets the
                                   //multiplication factor. If the value is outside of 4 and 20 (decimal), the PLL is disabled.
    let pump_75uA = 0b00 << 16; //00 (default) = the charge pump current is 75 μA
    let pump_100uA = 0b01 << 16; //01 (default) = the charge pump current is 100 μA
    let pump_125uA = 0b10 << 16; //10 (default) = the charge pump current is 125 μA
    let pump_150uA = 0b11 << 16; //11 (default) = the charge pump current is 150 μA
    let open1 = 0b0 << 15; //open
    let ppc_conf = 0b000 << 12; //The profile pin configuration bits control the configuration of the data and SDIO_x pins for the
                                //different modulation modes.
    let ru_rd = 0b00 << 10; //The RU/RD bits control the amplitude ramp-up/ramp-down time of a channel.
    let mod_lvl = (mod_level & 0b00) << 8; //00 = 2-level modulation
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
        | pump_150uA
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
    let fr2 = 0x02 << 16;

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

    (clear_phase_acc
        | fr2
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

enum AFPSelect {
    /// default: disable
    DisableModulation,
    AmpModulation,
    FreqModulation,
    PhaseModulation,
}

macro_rules! AFPS {
    ($s:expr) => {
        match $s {
            AFPSelect::DisableModulation => 0b00,
            AFPSelect::AmpModulation => 0b01,
            AFPSelect::FreqModulation => 0b10,
            AFPSelect::PhaseModulation => 0b11,
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

// TODO: decide the input type should be bool type, customized type or u8/u16/u32?
pub fn CFR(
    afp: AFPSelect,
    lsweep_nodwell: bool,
    lsweep_enable: bool,
    srr_at_ioupdate: bool,
) -> u32 {
    let cfr = 0x03 << 24;
    let AFP_select = AFPS!(afp) << 22;
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
pub fn CFTW(freq: u32) -> u32 {
    let cftw = 0x04 << 32;
    // LEARN: core_clock
    // LEARN: how to get Frequency Tuning Word (FTW) from desired output freq
    let cftw_value = freq2FTW0!(freq);
    cftw | cftw_value
}

pub fn CPOW(phase: u32) -> u16 {
    let cpow = 0x05 << 16;
    let open = 0b00 << 14;
    let cpow_value = ph2CPOW0!(phase);
    cpow | open | cpow_value
}

/// 24 bits + 1
/// amplitude ramp rate[23:16] default: N/A
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

pub fn LSRR(falling_sweep_ramp_rate: u32, rising_sweep_ramp_rate: u32) -> u16 {
    let lsrr = 0x07 << 16;
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
    lsrr | frr | rrr
}


// TODO: 匹配0b01, 0b10顺序有错误!!!
pub fn RDW(step: u32) -> u32 {
    let rdw = 0x08 << 32;
    let core_clock = sys_clk_rate!();
    let rdw_value = {if let Ok(afp) = AFP_SELECTOR.read() {
        match *afp {
            AFPSelector::NoModulation => {0}
            AFPSelector::Amplitude => {
                if step > 0x3ff {
                    log_func!(red:"amplitude modulation selected");
                    0x3ff << 22
                } else {
                    (step & 0x3ff )<< 22
                }
            }
            AFPSelector::Frequency => {
                // FIXME check  bitwise
                if step > core_clock {
                    log_func!(red:"frequency modulation selected");
                    (0xffffffff + 1) & 0xffffffff
                } else {
                    ((0xffffffff + 1) * step / core_clock) & 0xffffffff
                }
            }
            AFPSelector::PhaseDegree => {
                (((0x3fff + 1) * step / 360) & 0x3fff) << 18
            }
        }
    } else {
        log_func!(on_red:"RwLock<AFPSelector> is not accessed");
        0
    }};
    rdw | rdw_value
}

pub fn FDW(step: u32) -> u32 {}

pub fn CW(num: u8, word: u32) -> u8 {}

pub fn init_viaSPI() {}

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
