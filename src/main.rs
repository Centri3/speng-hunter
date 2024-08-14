mod handler;

use {
    anyhow::Result,
    handler::Handler,
    indoc::{formatdoc, writedoc},
    rand::Rng,
    std::{
        borrow::Cow,
        f32::consts::PI,
        fs::{self, File},
        io::{Read, Write},
        path::{Path, PathBuf},
        thread,
        time::{Duration, Instant},
    },
};

// Earth constants
const EARTH_MASS: f32 = 5.9724e+24f32;
// Good on you, Vladimir!
// const EARTH_EQUAT_RADIUS: f32 = 6378.14f32;
const EARTH_MEAN_RADIUS: f32 = 6371.0f32;
const EARTH_DENSITY: f32 = 5.51363f32;
const EARTH_GRAVITY: f32 = 9.80665f32;
const EARTH_ESC_VEL: f32 = 11.1784f32;
const EARTH_AVG_TEMP: f32 = 288.0f32;

// Address to the code of the selected object.
// Example: RS 0-3-397-1581-20880-7-556321-30 A3. Go visit it yourself! (:
const SELECTED_OBJECT_CODE: usize = 0x19a9e40usize;
// Pointer to the parameters of the selected object.
const SELECTED_OBJECT_POINTER: usize = 0x19a9ec0usize;
const SELECTED_SYSTEM_POINTER: usize = 0x19a9ec8usize;
const STAR_BROWSER_SYSTEMS_SEARCHED: usize = 0x1024114usize;
// Address to the number of Systems found.
const STAR_BROWSER_SYSTEMS_FOUND: usize = 0x1024118usize;
// Address to whether the Star browser is currently searching.
const STAR_BROWSER_SEARCHING: usize = 0x104a181usize;
// Address to the Star browser's current search radius.
const STAR_BROWSER_SEARCH_RADIUS: usize = 0x1024100usize;
const STAR_BROWSER_STAR_LIST_LEN: usize = 0x102410Cusize;
// Address to SE's current GUI scale.
const GUI_SCALE: usize = 0xe69434;

// Offsets from SELECTED_OBJECT_POINTER
const OBJECT_VOL_CLASS: usize = 0x34usize;
const OBJECT_BULK_CLASS: usize = 0x3cusize;
const OBJECT_MASS: usize = 0x11f8usize;
const OBJECT_EQUAT_RADIUS: usize = 0x1ca4usize;
const OBJECT_AVG_TEMP: usize = 0x1248usize;
const OBJECT_OBLATENESS: usize = 0x120cusize;
const OBJECT_LIFE: usize = 0x11bcusize;
const OBJECT_ATM_PRESSURE: usize = 0x17d8usize;
const OBJECT_HYDROSPHERE_DEPTH: usize = 0x15a0usize;
const OBJECT_HYDROSPHERE_ELEMENT_O2: usize = 0x14dcusize;
const OBJECT_HYDROSPHERE_SUM_OF_ELEMENTS: usize = 0x1590usize;
const OBJECT_BITFLAGS: usize = 0x11C0usize;
const GALAXY_TYPE: usize = 0x8usize;
const GALAXY_SIZE: usize = 0x20usize;

// Coordinates to some GUI elements.
// These are always the same whenever ran, despite being initialized at runtime.
const STAR_BROWSER_SEARCH_BUTTON: usize = 0x1025a78;
const STAR_BROWSER_CLEAR_BUTTON: usize = 0x1025d60;
const STAR_BROWSER_FILTER_TOGGLE: usize = 0x1029de8;
const STAR_BROWSER_FILTER_SORT: usize = 0x1027a70;

// Coordinates offsets
const GENERIC_OFFSET: i32 = 0xai32;
const FILTER_OFFSET: i32 = 0x6i32;
const SYSTEMS_OFFSET: i32 = 0x19i32;

fn main() {
    let handler = Handler::new();
    let mut rng = rand::thread_rng();

    // This is easier to write 1000 times.
    let base = handler.base();

    let lowest = (0..)
        .skip_while(|&i| {
            if i == 0 {
                Path::new("hunter.log").exists()
            } else {
                PathBuf::from(format!("hunter-{i}.log")).exists()
            }
        })
        .next()
        .unwrap();
    let mut log = File::create(&*if lowest == 0 {
        Cow::Borrowed("hunter.log")
    } else {
        Cow::Owned(format!("hunter-{lowest}.log"))
    })
    .unwrap();

    loop {
        // Select RG 0-3-397-1581, this is so we can reset the code of the currently
        // selected object. If we don't do this, it'll select nothing.
        handler.run_script("select_rg_397.se", "Select \"RG 0-3-397-1581\"".as_bytes());

        // Not entirely sure how long we need to sleep for, but hwe need to give SE time
        // to update the currently selected object (Or anything helse).
        thread::sleep(Duration::from_millis(50u64));

        'inner: loop {
            // Generate a random galaxy
            let level = rng.gen_range(1u32..9u32);
            let block = rng.gen_range(0u32..8u32.pow(level));
            let number = rng.gen_range(0u32..2500u32);

            // Write galaxy code to memory
            handler.write(level, base + SELECTED_OBJECT_CODE + 0x4);
            handler.write(block, base + SELECTED_OBJECT_CODE + 0x8);
            handler.write(number, base + SELECTED_OBJECT_CODE + 0x10);

            thread::sleep(Duration::from_millis(50u64));

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
            // let dist = rng.gen_range(0.0375f32..0.0625f32);
            let dist = rng.gen_range(0.25f32..0.325f32);

            handler.run_script(
                "goto.se",
                format!("Goto {{ Lat {} Lon {} Time 0 }}", 80.0f32, lat).as_bytes(),
            );

            thread::sleep(Duration::from_millis(200u64));

            // DistRad and Lat/Lon don't work together with Time 0, for some reason.
            handler.run_script(
                "goto_closer.se",
                format!("Goto {{ DistRad {} Time 0 }}", dist).as_bytes(),
            );

            thread::sleep(Duration::from_millis(200u64));

            // This is vile
            let gui_scale = handler.read::<f32>(base + GUI_SCALE);
            let search_button = (
                (handler.read::<f32>(base + STAR_BROWSER_SEARCH_BUTTON) * gui_scale) as i32
                    + GENERIC_OFFSET,
                (handler.read::<f32>(base + STAR_BROWSER_SEARCH_BUTTON + 0x4) * gui_scale) as i32
                    + GENERIC_OFFSET,
            );
            let clear_button = (
                (handler.read::<f32>(base + STAR_BROWSER_CLEAR_BUTTON) * gui_scale) as i32
                    + GENERIC_OFFSET,
                (handler.read::<f32>(base + STAR_BROWSER_CLEAR_BUTTON + 0x4) * gui_scale) as i32
                    + GENERIC_OFFSET,
            );
            let filter_toggle = (
                (handler.read::<f32>(base + STAR_BROWSER_FILTER_TOGGLE) * gui_scale) as i32
                    + GENERIC_OFFSET,
                (handler.read::<f32>(base + STAR_BROWSER_FILTER_TOGGLE + 0x4) * gui_scale) as i32
                    + GENERIC_OFFSET,
            );
            let filter_sort = (
                (handler.read::<f32>(base + STAR_BROWSER_FILTER_SORT) * gui_scale) as i32
                    + GENERIC_OFFSET,
                (handler.read::<f32>(base + STAR_BROWSER_FILTER_SORT + 0x4) * gui_scale) as i32
                    + GENERIC_OFFSET,
            );

            // Click clear button 3 times, otherwise it'll sometimes not clear the search
            // properly

            for _ in 0u32..=2u32 {
                handler.click(clear_button.0, clear_button.1);
            }

            thread::sleep(Duration::from_millis(160u64));

            // Click search button

            handler.click(search_button.0, search_button.1);

            let start = Instant::now();
            let mut systems_searched = handler.read::<i32>(base + STAR_BROWSER_SYSTEMS_SEARCHED);
            let mut systems_found = handler.read::<i32>(base + STAR_BROWSER_SYSTEMS_FOUND);

            while handler.read::<u32>(base + STAR_BROWSER_SEARCHING) == 0u32 {
                systems_searched = handler.read::<i32>(base + STAR_BROWSER_SYSTEMS_SEARCHED);
                systems_found = handler.read::<i32>(base + STAR_BROWSER_SYSTEMS_FOUND);

                if start.elapsed() > Duration::from_secs(6u64) && systems_searched == 0 {
                    handler.click(clear_button.0, clear_button.1);
                    break;
                }

                if start.elapsed() > Duration::from_secs(80u64) {
                    handler.click(clear_button.0, clear_button.1);

                    break;
                }
            }

            // Double-click filter toggle

            for _ in 0u32..=1u32 {
                handler.click(filter_toggle.0, filter_toggle.1)
            }

            // Check each system
            for i in 0i32..=i32::min(systems_found - 1i32, 3i32) {
                handler.click(
                    filter_sort.0 - 0x40,
                    filter_sort.1 + (i + 1i32) * SYSTEMS_OFFSET,
                );

                // Reinitialize selected_object. For some reason, some objects refuse to give
                // their parameters sometimes, so retry until it works.
                for _ in 0u32..100u32 {
                    selected_object = handler.read::<usize>(base + SELECTED_OBJECT_POINTER);

                    if selected_object != 0usize {
                        break;
                    }
                }

                thread::sleep(Duration::from_millis(100));

                handler.click(
                    filter_sort.0,
                    filter_sort.1 + FILTER_OFFSET + (i + 1i32) * SYSTEMS_OFFSET,
                );

                // Reinitialize selected_object. For some reason, some objects refuse to give
                // their parameters sometimes, so retry until it works.
                for _ in 0u32..100u32 {
                    selected_object = handler.read::<usize>(base + SELECTED_OBJECT_POINTER);

                    if selected_object != 0usize {
                        break;
                    }
                }

                // This is also vile
                let vol_class = handler.read::<u32>(selected_object + OBJECT_VOL_CLASS);
                let bulk_class = handler.read::<u32>(selected_object + OBJECT_BULK_CLASS);
                let mass = handler.read::<f32>(selected_object + OBJECT_MASS) * EARTH_MASS;
                let equat_radius = handler.read::<f32>(selected_object + OBJECT_EQUAT_RADIUS);
                let avg_temp = handler.read::<f32>(selected_object + OBJECT_AVG_TEMP);
                let oblateness = handler.read::<f32>(selected_object + OBJECT_OBLATENESS);
                let life = handler.read::<u32>(selected_object + OBJECT_LIFE);
                let atm_pressure = handler.read::<f32>(selected_object + OBJECT_ATM_PRESSURE);
                let hydrosphere_depth =
                    handler.read::<f32>(selected_object + OBJECT_HYDROSPHERE_DEPTH);
                let hydrosphere_element_o2 =
                    handler.read::<f32>(selected_object + OBJECT_HYDROSPHERE_ELEMENT_O2);
                let hydrosphere_sum_of_elements =
                    handler.read::<f32>(selected_object + OBJECT_HYDROSPHERE_SUM_OF_ELEMENTS);
                let is_a = match handler.read::<u32>(selected_object + OBJECT_BITFLAGS)
                    & 0b00000010000000000000000000000000
                {
                    0b00000010000000000000000000000000 => true,
                    _ => false,
                };
                let is_b = match handler.read::<u32>(selected_object + OBJECT_BITFLAGS)
                    & 0b00000100000000000000000000000000
                {
                    0b00000100000000000000000000000000 => true,
                    _ => false,
                };
                let is_binary = is_a | is_b;
                let b_vol_class = handler.read::<u32>(selected_object + 0x36D0 + OBJECT_VOL_CLASS);
                let b_life = handler.read::<u32>(selected_object + 0x36D0 + OBJECT_LIFE);
                let system = handler.read::<usize>(base + SELECTED_SYSTEM_POINTER);
                let seed = handler.read::<u32>(system + 0x170);
                let num_planets = handler.read::<u32>(system + 0xB4);

                let polar_radius = equat_radius * (1.0f32 - oblateness);
                let mean_radius = f32::cbrt(equat_radius.powi(2i32) * polar_radius);
                let gravity = (mass / EARTH_MASS) / (mean_radius / EARTH_MEAN_RADIUS).powi(2i32)
                    * EARTH_GRAVITY;

                let density = mass * 1.0e-12f32 / (4.0f32 / 3.0f32 * PI * mean_radius.powi(3i32));
                let esc_vel = f32::sqrt(2.0f32 * gravity * mean_radius * 1000.0f32) * 0.001f32;

                let n = 1.0f32 / 4.0f32;

                // Vile. Again. Also, this isn't 100% accurate, but it's way more than close
                // enough.
                let esi = f32::powf(
                    1.0f32
                        - ((mean_radius - EARTH_MEAN_RADIUS) / (mean_radius + EARTH_MEAN_RADIUS))
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

                let code = get_code(&handler);
                let mut body = vec!["RARE"];

                let high_esi_earthlike = esi > 0.9895f32
                    && (life == 1703936u32 || life == 1075445760u32)
                    && vol_class == 3u32;
                try_add_body(&mut body, high_esi_earthlike, "HIGH_ESI_EARTHLIKE");
                let high_esi_minigiant = esi > 0.9875 && bulk_class == 5u32;
                try_add_body(&mut body, high_esi_minigiant, "HIGH_ESI_MINIGIANT");
                if !high_esi_earthlike && !high_esi_minigiant {
                    try_add_body(&mut body, esi > 0.9985, "HIGH_ESI")
                };
                try_add_body(
                    &mut body,
                    (0.999995..=1.00005).contains(&(mass / EARTH_MASS))
                        && (6370.97f32..=6371.31f32).contains(&mean_radius),
                    "1M1R",
                );
                try_add_body(
                    &mut body,
                    is_a && (b_life == 1703936u32 || b_life == 1075445760u32)
                        && b_vol_class == 3u32,
                    "BINARY_EARTHLIKES",
                );
                try_add_body(
                    &mut body,
                    mass / EARTH_MASS > 60.0 && atm_pressure < 1000.0,
                    "HYPERTERRESTRIAL",
                );
                try_add_body(
                    &mut body,
                    mass / EARTH_MASS > 25537.0 && atm_pressure > 1000.0,
                    "AB_FLIP",
                );
                try_add_body(
                    &mut body,
                    hydrosphere_depth > 6.0f32
                        && f32::max(hydrosphere_element_o2, 0.0001) / hydrosphere_sum_of_elements
                            > 0.2f32,
                    "O2_OCEANS",
                );
                try_add_body(
                    &mut body,
                    mass / EARTH_MASS < 0.0009 && (2..=8).contains(&vol_class),
                    "SMALL_NONARID",
                );

                if let [rare_prefix] = &mut *body {
                    *rare_prefix = "COMMON";
                }

                writedoc!(
                    log,
                    "
                    {}: {code}
                    -----
                    Earth masses:               {}M🜨
                    Jupiter masses:             {}M♃
                    Solar masses:               {}M☉
                    -----
                    Equatorial radius:          {equat_radius} km
                    Mean radius:                {mean_radius} km
                    Polar radius:               {polar_radius} km
                    -----
                    Average temperature:        {avg_temp} K
                    -----
                    Raw life:                   0b{life}
                    -----
                    Atmospheric pressure:       {atm_pressure}
                    -----
                    Volatiles class:            \"{}\"
                    Hydrosphere maximum depth:  {hydrosphere_depth}
                    O2:                         {}%
                    -----
                    Density:                    {density} g/cm³
                    Escape velocity:            {esc_vel} km/sec
                    Gravity:                    {gravity} m/sec²
                    -----
                    Earth Similarity Index:     {esi}
                    -----
                    System's seed:              {seed}
                    System's number of planets: {num_planets}


                    ",
                    body.join(" + "),
                    mass / EARTH_MASS,
                    mass / EARTH_MASS / 317.82838,
                    mass / EARTH_MASS / 322946.0,
                    get_volatiles_class(vol_class),
                    hydrosphere_element_o2 / hydrosphere_sum_of_elements * 100.0,
                )
                .unwrap();
            }
            break 'inner;
        }
    }
}

fn get_volatiles_class(v: u32) -> &'static str {
    match v {
        0 => "airless",
        1 => "arid",
        2 => "lacustrine",
        3 => "marine",
        4 => "oceanic",
        5 => "superoceanic",

        // unused
        6 => "glacial",
        7 => "superglacial",

        // implementation details
        8 => "non-arid",
        9 => "",
        _ => "any",
    }
}

fn get_code(handler: &Handler) -> String {
    handler.run_script("get_code.se", "PrintNames true".as_bytes());

    thread::sleep(Duration::from_millis(200));

    let mut path = handler.exe.as_path().to_path_buf();
    path.set_file_name("se.log");

    fs::read_to_string(path)
        .unwrap()
        .rsplit_once("Body full def:")
        .unwrap()
        .1
        .lines()
        .next()
        .unwrap()
        .to_owned()
        .trim()
        .to_owned()
}

fn try_add_body(body: &mut Vec<&'static str>, cond: bool, elem: &'static str) {
    if cond {
        body.push(elem);
    }
}
