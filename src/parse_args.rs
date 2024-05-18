const HELP: &str = "\
The Fabulous Agon Emulator!

USAGE:
  fab-agon-emulator [OPTIONS]

OPTIONS:
  -d, --debugger        Enable the eZ80 debugger
  -b, --breakpoint      Set a breakpoint before starting
  -f, --fullscreen      Start in fullscreen mode
  -h, --help            Prints help information
  -u, --unlimited-cpu   Don't limit eZ80 CPU frequency
  --firmware 1.03       Use quark 1.03 firmware (default is console8)
  --firmware quark      Use quark 1.04 firmware (default is console8)
  --firmware electron   Use ElectronOS firmware (default is console8)
  --mode <n>            Start in a specific screen mode
  --sdcard <path>       Sets the path of the emulated SDCard
  --scale 4:3           (default) Scale Agon screen to 4:3 aspect ratio
  --scale integer       Scale Agon screen to an integer multiple
  --scale stretch       Scale Agon screen to full window size
  --border #rrggbb      Colour of border around Agon screen (default #000000)

ADVANCED:
  --mos PATH            Use a different MOS.bin firmware
  --vdp PATH            Use a different VDP dll/so firmware
  --renderer sw         Use software renderer
  --renderer hw         Use GL/D3D renderer (default)
  --uart1-device <dev>  Link ez80 uart1 to this host serial device
  --uart1-baud <rate>   Open --uart1-device with the given baud rate
  --verbose             Verbose mode (includes VDP debug logs)
  --ralt-hostkey        Use right-alt (AltGr) as the emulator host key
  -z, --zero            Initialize ram with zeroes instead of random values
";

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum FirmwareVer {
    quark103,
    quark,
    console8,
    electron,
}

#[derive(Debug)]
pub enum Renderer {
    Software,
    Accelerated,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ScreenScale {
    StretchAny,
    Scale4_3,
    ScaleInteger,
}

#[derive(Debug)]
pub struct AppArgs {
    pub sdcard: Option<String>,
    pub debugger: bool,
    pub breakpoint: Option<String>,
    pub unlimited_cpu: bool,
    pub fullscreen: bool,
    pub verbose: bool,
    pub zero: bool,
    pub scr_mode: Option<u32>,
    pub mos_bin: Option<std::path::PathBuf>,
    pub vdp_dll: Option<std::path::PathBuf>,
    pub firmware: FirmwareVer,
    pub screen_scale: ScreenScale,
    pub renderer: Renderer,
    pub border: u32,
    pub uart1_device: Option<String>,
    pub uart1_baud: Option<u32>,
    pub alternative_hostkey: bool,
}

pub fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // for `make install`
    if pargs.contains("--prefix") {
        print!("{}", option_env!("PREFIX").unwrap_or(""));
        std::process::exit(0);
    }

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let firmware_ver: Option<String> = pargs.opt_value_from_str("--firmware")?;
    let renderer: Option<String> = pargs.opt_value_from_str("--renderer")?;
    let scale: Option<String> = pargs.opt_value_from_str("--scale")?;
    let border: String = pargs
        .opt_value_from_str("--border")?
        .unwrap_or("0".to_string());

    let args = AppArgs {
        sdcard: pargs.opt_value_from_str("--sdcard")?,
        debugger: pargs.contains(["-d", "--debugger"]),
        breakpoint: pargs.opt_value_from_str(["-b", "--breakpoint"])?,
        unlimited_cpu: pargs.contains(["-u", "--unlimited_cpu"]),
        fullscreen: pargs.contains(["-f", "--fullscreen"]),
        alternative_hostkey: pargs.contains("--ralt-hostkey"),
        verbose: pargs.contains("--verbose"),
        zero: pargs.contains(["-z", "--zero"]),
        scr_mode: pargs.opt_value_from_str("--mode")?,
        border: match u32::from_str_radix(border.as_str(), 16) {
            Ok(v) => v,
            Err(_) => {
                println!("Error parsing --border colour. Expected hex colour, eg #ff0070");
                std::process::exit(0);
            }
        },
        screen_scale: match scale.unwrap_or("4:3".to_string()).as_str() {
            "4:3" => ScreenScale::Scale4_3,
            "stretch" => ScreenScale::StretchAny,
            "integer" => ScreenScale::ScaleInteger,
            s => {
                println!(
                    "Unknown --scale value: {}. Valid values are: 4:3, integer, stretch",
                    s
                );
                std::process::exit(0);
            }
        },
        mos_bin: pargs.opt_value_from_str("--mos")?,
        vdp_dll: pargs.opt_value_from_str("--vdp")?,
        uart1_device: pargs.opt_value_from_str("--uart1-device")?,
        uart1_baud: pargs.opt_value_from_str("--uart1-baud")?,
        renderer: if let Some(r) = renderer {
            match r.as_str() {
                "hw" => Renderer::Accelerated,
                "sw" => Renderer::Software,
                _ => {
                    println!("Unknown --renderer value: {}. Valid values are: hw, sw", r);
                    std::process::exit(0);
                }
            }
        } else {
            Renderer::Accelerated
        },
        firmware: if let Some(ver) = firmware_ver {
            if ver == "1.03" {
                FirmwareVer::quark103
            } else if ver == "quark" {
                FirmwareVer::quark
            } else if ver == "console8" {
                FirmwareVer::console8
            } else if ver == "electron" {
                FirmwareVer::electron
            } else {
                println!("Unknown --firmware value: {}. Valid values are: 1.03, quark, console8, electron", ver);
                std::process::exit(0);
            }
        } else {
            FirmwareVer::console8
        },
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}
