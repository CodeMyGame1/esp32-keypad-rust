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
use heapless::Deque;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]

fn run_keypad(p: Peripherals) -> ! {
    let output_config
        = OutputConfig::default().with_drive_mode(DriveMode::OpenDrain);
    let input_config
        = InputConfig::default().with_pull(Pull::Up);

    // Connector pins 1-4 are physical columns, driven one at a time.
    let mut driven_columns = [
        Output::new(p.GPIO13, Level::High, output_config),
        Output::new(p.GPIO12, Level::High, output_config),
        Output::new(p.GPIO14, Level::High, output_config),
        Output::new(p.GPIO27, Level::High, output_config),
    ];

    // Connector pins 5-8 are physical rows, read as inputs.
    let sensed_rows = [
        Input::new(p.GPIO26, input_config),
        Input::new(p.GPIO25, input_config),
        Input::new(p.GPIO33, input_config),
        Input::new(p.GPIO32, input_config),
    ];

    // Index by driven physical column, then sensed physical row.
    let keys = [
        ['1', '4', '7', '*'],
        ['2', '5', '8', '0'],
        ['3', '6', '9', '#'],
        ['A', 'B', 'C', 'D'],
    ];

    let delay = Delay::new();

    // Used for the core keypad loop.
    let mut candidate: Option<(usize, usize)> = None;
    let mut stable: Option<(usize, usize)> = None;
    let mut matching_scans: u8 = 0;

    // i love state machines
    let mut keypad_queue: Deque<char, 32> = Deque::new();

    log::info!("Keypad ready!");

    loop {
        let mut detected = None;
        let mut pressed_count = 0;

        for column in 0..4 {
            // LOW activates this column, HIGH releases it.
            driven_columns[column].set_low();
            delay.delay_micros(10);

            for row in 0..4 {
                if sensed_rows[row].is_low() {
                    detected = Some((column, row));
                    pressed_count += 1;
                }
            }

            driven_columns[column].set_high();
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

            if let Some((column, row)) = stable {
                let key: char = keys[column][row];

                log::info!(
                    "Pressed: {} (connector pins {} and {})",
                    key,
                    column + 1,
                    row + 5
                );

                if let Err(_value) = keypad_queue.push_back(key) {
                    log::error!("CRITICAL ERROR: KEYPAD QUEUE FULL, INPUT DROPPED.");
                }
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
