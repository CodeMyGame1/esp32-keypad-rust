#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{DriveMode, Input, InputConfig, Level, Output, OutputConfig, Pull},
    main,
    peripherals::Peripherals,
};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]

fn run_keypad(p: Peripherals) -> ! {
    let row_config
        = OutputConfig::default().with_drive_mode(DriveMode::OpenDrain);
    let col_config
        = InputConfig::default().with_pull(Pull::Up);

    let mut rows = [
        Output::new(p.GPIO13, Level::High, row_config),
        Output::new(p.GPIO14, Level::High, row_config),
        Output::new(p.GPIO25, Level::High, row_config),
        Output::new(p.GPIO26, Level::High, row_config),
    ];

    let cols = [
        Input::new(p.GPIO27, col_config),
        Input::new(p.GPIO32, col_config),
        Input::new(p.GPIO33, col_config),
        Input::new(p.GPIO23, col_config),
    ];

    let keys = [
        ['1', '2', '3', 'A'],
        ['4', '5', '6', 'B'],
        ['7', '8', '9', 'C'],
        ['*', '0', '#', 'D'],
    ];

    let delay = Delay::new();

    let mut candidate: Option<char> = None;
    let mut stable: Option<char> = None;
    let mut matching_scans: u8 = 0;

    log::info!("Keypad ready!");

    loop {
        let mut detected = None;
        let mut pressed_count = 0;

        for r in 0..4 {
            // LOW activates this row, HIGH releases it.
            rows[r].set_low();
            delay.delay_micros(10);

            for c in 0..4 {
                if cols[c].is_low() {
                    detected = Some(keys[r][c]);
                    pressed_count += 1;
                }
            }

            rows[r].set_high();
        }

        // Accept only one key at a time.
        if pressed_count != 1 {
            detected = None;
        }

        // Require several matching readings to debounce.
        if detected != candidate {
            candidate = detected;
            matching_scans = 0;
        } else {
            matching_scans = matching_scans.saturating_add(1);
        }

        if (matching_scans >= 5) && (candidate != stable) {
            stable = candidate;

            if let Some(key) = stable {
                log::info!("Pressed: {}", key);

                // TODO: Button handling code here.
            }
        }

        delay.delay_millis(5);
    }
}

#[main]
fn main() -> ! {
    // generator version: 1.4.0
    // generator parameters: -o esp32 -o vscode -o log -o esp-backtrace

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    loop {
        run_keypad(_peripherals);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}
