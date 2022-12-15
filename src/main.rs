// Note that you MUST build on Rust 1.63.0!!! Enigo SUCKS (not really) and is
// broken on 1.64.0 and above but I don't want to rewrite for rdev or similar...

mod handler;

use {
    enigo::{Enigo, Key, KeyboardControllable, MouseButton, MouseControllable},
    handler::Handler,
    rand::Rng,
    std::{
        f32::consts::PI,
        thread,
        time::{Duration, Instant},
    },
};

// Earth constants
const EARTH_MASS: f32 = 5.9724e+24f32;
const EARTH_EQUAT_RADIUS: f32 = 6378.14f32;
const EARTH_MEAN_RADIUS: f32 = 6371.0f32;
const EARTH_DENSITY: f32 = 5.51363f32;
const EARTH_GRAVITY: f32 = 9.80665f32;
const EARTH_ESC_VEL: f32 = 11.1784f32;
const EARTH_AVG_TEMP: f32 = 288.0f32;

// Address to the code of the selected object.
// Example: RS 0-3-397-1581-20880-7-556321-30 A3. Go visit it yourself! (:
const SELECTED_OBJECT_CODE: usize = 0x19a9ea0usize;
// Pointer to the parameters of the selected object.
const SELECTED_OBJECT_POINTER: usize = 0x19a9f20usize;
// Address to the number of Systems found.
const STAR_BROWSER_SYSTEMS_FOUND: usize = 0x1024178usize;
// Address to whether the Star browser is currently searching.
const STAR_BROWSER_SEARCHING: usize = 0x104a1e1usize;
// Address to SE's current GUI scale.
const GUI_SCALE: usize = 0xe69494;

// Offsets from SELECTED_OBJECT_POINTER
const OBJECT_CLASS: usize = 0x34usize;
const OBJECT_MASS: usize = 0x11f8usize;
const OBJECT_EQUAT_RADIUS: usize = 0x1ca4usize;
const OBJECT_AVG_TEMP: usize = 0x1248usize;
const OBJECT_OBLATENESS: usize = 0x120cusize;
const OBJECT_LIFE: usize = 0x11bcusize;
const OBJECT_ATM_PRESSURE: usize = 0x17d8usize;
const GALAXY_TYPE: usize = 0x8usize;
const GALAXY_SIZE: usize = 0x20usize;

// Coordinates to some GUI elements.
// These are always the same whenever ran, despite being initialized at runtime.
const STAR_BROWSER_SEARCH_BUTTON: usize = 0x1025ad8;
const STAR_BROWSER_CLEAR_BUTTON: usize = 0x1025dc0;
const STAR_BROWSER_FILTER_TOGGLE: usize = 0x1029e48;
const STAR_BROWSER_FILTER_SORT: usize = 0x1027ad0;

// Coordinates offsets
const GENERIC_OFFSET: i32 = 0xai32;
const FILTER_OFFSET: i32 = 0x6i32;
const SYSTEMS_OFFSET: i32 = 0x19i32;
const WINDOWED_OFFSET: i32 = 0x14i32;

fn main() {
    let handler = Handler::new();
    let mut rng = rand::thread_rng();
    let mut enigo = Enigo::new();

    // This is easier to write 1000 times.
    let base = handler.base();

    loop {
        // Select RG 0-3-397-1581, this is so we can reset the code of the currently
        // selected object. If we don't do this, it'll select nothing.
        handler.run_script("select_rg_397.se", "Select \"RG 0-3-397-1581\"".as_bytes());

        // Not entirely sure how long we need to sleep for, but we need to give SE time
        // to update the currently selected object (Or anything else).
        thread::sleep(Duration::from_millis(160u64));
        // Generate a random galaxy

        'inner: loop {
            let level = rng.gen_range(1u32..9u32);
            let block = rng.gen_range(0u32..8u32.pow(level));
            let number = rng.gen_range(0u32..2500u32);

            // Write galaxy code to memory
            handler.write(level, base + SELECTED_OBJECT_CODE + 0x4);
            handler.write(block, base + SELECTED_OBJECT_CODE + 0x8);
            handler.write(number, base + SELECTED_OBJECT_CODE + 0x10);

            thread::sleep(Duration::from_millis(160u64));

            let mut selected_object = handler.read::<usize>(base + SELECTED_OBJECT_POINTER);

            // This could mean that the galaxy doesn't exist, or my code is too fast. Skip.
            // Also, skip any galaxies with a type of E/Irr or isn't max size
            if selected_object == 0usize
                || (1u32..=8u32).contains(&handler.read::<u32>(selected_object + GALAXY_TYPE))
                || handler.read::<u32>(selected_object + GALAXY_TYPE) == 16u32
                || handler.read::<f32>(selected_object + GALAXY_SIZE) != 50000.0f32
            {
                continue;
            }

            let lat = rng.gen_range(-180.0f32..180.0f32);
            // todo!(); if Systems found reducing is fixed, then up max to 0.625. Currently
            // stars aren't dense enough that far out for 100K systems to work
            let dist = rng.gen_range(0.0625f32..0.125f32);

            handler.run_script(
                "goto.se",
                format!("Goto {{ Lat {} Lon {} Time 0 }}", 90.0f32, lat).as_bytes(),
            );

            thread::sleep(Duration::from_millis(160u64));

            // DistRad and Lat/Lon don't work together with Time 0, for some reason.
            handler.run_script(
                "goto_closer.se",
                format!("Goto {{ DistRad {} Time 0 }}", dist).as_bytes(),
            );

            // This is vile
            let gui_scale = handler.read::<f32>(base + GUI_SCALE);
            let search_button = (
                (handler.read::<f32>(base + STAR_BROWSER_SEARCH_BUTTON) * gui_scale) as i32
                    + GENERIC_OFFSET,
                (handler.read::<f32>(base + STAR_BROWSER_SEARCH_BUTTON + 0x4) * gui_scale) as i32
                    + GENERIC_OFFSET
                    + WINDOWED_OFFSET,
            );
            let clear_button = (
                (handler.read::<f32>(base + STAR_BROWSER_CLEAR_BUTTON) * gui_scale) as i32
                    + GENERIC_OFFSET,
                (handler.read::<f32>(base + STAR_BROWSER_CLEAR_BUTTON + 0x4) * gui_scale) as i32
                    + GENERIC_OFFSET
                    + WINDOWED_OFFSET,
            );
            let filter_toggle = (
                (handler.read::<f32>(base + STAR_BROWSER_FILTER_TOGGLE) * gui_scale) as i32
                    + GENERIC_OFFSET,
                (handler.read::<f32>(base + STAR_BROWSER_FILTER_TOGGLE + 0x4) * gui_scale) as i32
                    + GENERIC_OFFSET
                    + WINDOWED_OFFSET,
            );
            let filter_sort = (
                (handler.read::<f32>(base + STAR_BROWSER_FILTER_SORT) * gui_scale) as i32
                    + GENERIC_OFFSET,
                (handler.read::<f32>(base + STAR_BROWSER_FILTER_SORT + 0x4) * gui_scale) as i32
                    + GENERIC_OFFSET
                    + WINDOWED_OFFSET,
            );

            // Click clear button 3 times, otherwise it'll sometimes not clear the search
            // properly

            for _ in 0u32..=2u32 {
                enigo.mouse_move_to(clear_button.0, clear_button.1);

                enigo.mouse_click(MouseButton::Left);

                thread::sleep(Duration::from_millis(160u64));
            }

            // Click search button

            enigo.mouse_move_to(search_button.0, search_button.1);

            enigo.mouse_click(MouseButton::Left);

            thread::sleep(Duration::from_millis(160u64));

            let start = Instant::now();
            let mut systems_found = handler.read::<i32>(base + STAR_BROWSER_SYSTEMS_FOUND);

            while handler.read::<u32>(base + STAR_BROWSER_SEARCHING) == 0u32 {
                systems_found = handler.read::<i32>(base + STAR_BROWSER_SYSTEMS_FOUND);

                // Stop waiting after 180s or once Systems found > 22.
                if start.elapsed() == Duration::from_secs(180u64) || systems_found > 22i32 {
                    enigo.mouse_move_to(clear_button.0, clear_button.1);

                    enigo.mouse_click(MouseButton::Left);

                    thread::sleep(Duration::from_millis(160u64));

                    break;
                }
            }

            // Double-click filter toggle

            enigo.mouse_move_to(filter_toggle.0, filter_toggle.1);

            for _ in 0u32..=1u32 {
                enigo.mouse_click(MouseButton::Left)
            }

            thread::sleep(Duration::from_millis(160u64));

            // Move to filter

            enigo.mouse_move_to(filter_sort.0, filter_sort.1);

            thread::sleep(Duration::from_millis(160u64));

            // Check each system
            for i in 0i32..=i32::min(systems_found - 1i32, 21i32) {
                enigo.mouse_move_to(
                    filter_sort.0,
                    filter_sort.1 + FILTER_OFFSET + (i + 1i32) * SYSTEMS_OFFSET,
                );

                enigo.mouse_click(MouseButton::Left);

                thread::sleep(Duration::from_millis(160u64));

                // Reinitialize selected_object. For some reason, some objects refuse to give
                // their parameters sometimes, so retry until it works.
                for _ in 0u32..100u32 {
                    selected_object = handler.read::<usize>(base + SELECTED_OBJECT_POINTER);

                    if selected_object != 0usize {
                        break;
                    }

                    thread::sleep(Duration::from_millis(10))
                }

                // This is also vile
                let class = handler.read::<u32>(selected_object + OBJECT_CLASS);
                let mass = handler.read::<f32>(selected_object + OBJECT_MASS) * EARTH_MASS;
                let equat_radius = handler.read::<f32>(selected_object + OBJECT_EQUAT_RADIUS);
                let avg_temp = handler.read::<f32>(selected_object + OBJECT_AVG_TEMP);
                let oblateness = handler.read::<f32>(selected_object + OBJECT_OBLATENESS);
                let life = handler.read::<u32>(selected_object + OBJECT_LIFE);
                let atm_pressure = handler.read::<f32>(selected_object + OBJECT_ATM_PRESSURE);

                let polar_radius = equat_radius * (1.0f32 - oblateness);
                let mean_radius = f32::cbrt(equat_radius.powi(2i32) * polar_radius);
                let gravity = (mass / EARTH_MASS) / (mean_radius / EARTH_MEAN_RADIUS).powi(2i32)
                    * EARTH_GRAVITY;

                let density = mass * 1.0e-12f32 / (4.0f32 / 3.0f32 * PI * mean_radius.powi(3i32));
                let esc_vel = f32::sqrt(2.0f32 * gravity * equat_radius * 1000.0f32) * 0.001f32;

                let n = 1.0f32 / 4.0f32;

                // Vile. Again. Also, this isn't 100% accurate, but it's way more than close
                // enough.
                let esi = f32::powf(
                    1.0f32
                        - ((equat_radius - EARTH_EQUAT_RADIUS)
                            / (equat_radius + EARTH_EQUAT_RADIUS))
                            .abs(),
                    0.57f32 * n,
                ) * f32::powf(
                    1.0f32 - ((density - EARTH_DENSITY) / (density + EARTH_DENSITY)).abs(),
                    1.07f32 * n,
                ) * f32::powf(
                    1.0f32 - ((esc_vel - EARTH_ESC_VEL) / (esc_vel + EARTH_ESC_VEL)).abs(),
                    0.70f32 * n,
                ) * f32::powf(
                    1.0f32 - ((avg_temp - EARTH_AVG_TEMP) / (avg_temp + EARTH_AVG_TEMP)).abs(),
                    5.58f32 * n,
                );

                // 0.998+ ESI,
                // 1 mass 1 radius,
                // 0.988+ ESI minigiant,
                // decent Hyperterrestrial,
                // decent A|B Flip,
                // and 0.990+ ESI Earth-like.
                if esi > 0.9975f32
                    || (0.999995f32..1.00005f32).contains(&mass)
                        && (6370.97f32..6371.31f32).contains(&equat_radius)
                    || esi > 0.9875 && atm_pressure > 1000.0f32
                    || mass / EARTH_MASS > 60.0f32 && atm_pressure < 1000.0f32
                    || mass / EARTH_MASS > 25537.0f32 && atm_pressure > 1000.0f32
                    || esi > 0.9895f32
                        && (life == 1703936u32 || life == 1075445760u32)
                        && class == 3u32
                {
                    enigo.key_down(Key::Control);
                    enigo.key_click(Key::F12);
                    enigo.key_up(Key::Control);
                }

                enigo.key_click(Key::Layout('h'));
            }

            break 'inner;
        }
    }
}
