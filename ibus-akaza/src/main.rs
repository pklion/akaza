#![allow(non_upper_case_globals)]

extern crate alloc;

use std::ffi::{c_char, c_void, CStr};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::{thread, time};

use anyhow::Result;
use clap::Parser;
use log::{error, info, warn};

use ibus_sys::core::ibus_main;
use ibus_sys::engine::IBusEngine;
use ibus_sys::glib::{gchar, guint};
use libakaza::config::Config;
use libakaza::engine::bigram_word_viterbi_engine::BigramWordViterbiEngineBuilder;
use libakaza::user_side_data::user_data::UserData;

use crate::context::AkazaContext;
use crate::wrapper_bindings::{ibus_akaza_init, ibus_akaza_set_callback};

mod commands;
mod context;
mod input_mode;
mod keymap;
mod wrapper_bindings;

unsafe extern "C" fn process_key_event(
    context: *mut c_void,
    engine: *mut IBusEngine,
    keyval: guint,
    keycode: guint,
    modifiers: guint,
) -> bool {
    let context_ref = &mut *(context as *mut AkazaContext);
    context_ref.process_key_event(engine, keyval, keycode, modifiers)
}

unsafe extern "C" fn candidate_clicked(
    context: *mut c_void,
    engine: *mut IBusEngine,
    index: guint,
    button: guint,
    state: guint,
) {
    let context_ref = &mut *(context as *mut AkazaContext);
    context_ref.do_candidate_clicked(engine, index, button, state);
}

unsafe extern "C" fn focus_in(context: *mut c_void, engine: *mut IBusEngine) {
    let context_ref = &mut *(context as *mut AkazaContext);
    context_ref.do_focus_in(engine);
}

unsafe extern "C" fn property_activate(
    context: *mut c_void,
    engine: *mut IBusEngine,
    prop_name: *mut gchar,
    prop_state: guint,
) {
    let context_ref = &mut *(context as *mut AkazaContext);
    context_ref.do_property_activate(
        engine,
        CStr::from_ptr(prop_name as *mut c_char)
            .to_string_lossy()
            .to_string(),
        prop_state,
    );
}

fn load_user_data() -> Arc<Mutex<UserData>> {
    match UserData::load_from_default_path() {
        Ok(user_data) => Arc::new(Mutex::new(user_data)),
        Err(err) => {
            error!("Cannot load user data: {}", err);
            Arc::new(Mutex::new(UserData::default()))
        }
    }
}

#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None)]
struct IBusAkazaArgs {
    #[clap(long)]
    ibus: bool,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() -> Result<()> {
    let arg: IBusAkazaArgs = IBusAkazaArgs::parse();

    env_logger::Builder::new()
        .filter_level(arg.verbose.log_level_filter())
        .init();

    info!("Starting ibus-akaza(rust version)");

    unsafe {
        let sys_time = SystemTime::now();
        let user_data = load_user_data();
        let akaza = BigramWordViterbiEngineBuilder::new(Config::load()?)
            .user_data(user_data.clone())
            .load_user_config(true)
            .build()?;
        let mut ac = AkazaContext::new(akaza);
        let new_sys_time = SystemTime::now();
        let difference = new_sys_time.duration_since(sys_time)?;
        info!(
            "Initialized ibus-akaza engine in {} milliseconds.",
            difference.as_millis()
        );

        // ユーザー辞書をバックグラウンドで保存するスレッド。
        thread::Builder::new()
            .name("user-data-save-thread".to_string())
            .spawn(move || {
                let interval = time::Duration::from_secs(60);

                // スレッド内で雑に例外投げるとスレッドとまっちゃうので丁寧めに処理する。
                loop {
                    if let Ok(mut data) = user_data.lock() {
                        if let Err(e) = data.write_user_stats_file() {
                            warn!("Cannot save user stats file: {}", e);
                        }
                    } else {
                        warn!("Cannot get mutex for saving user data")
                    };
                    thread::sleep(interval);
                }
            })?;

        ibus_akaza_set_callback(
            &mut ac as *mut _ as *mut c_void,
            process_key_event,
            candidate_clicked,
            focus_in,
            property_activate,
        );

        ibus_akaza_init(arg.ibus);

        info!("Enter the ibus_main()");

        // run main loop
        ibus_main();

        warn!("Should not reach here.");
    }
    Ok(())
}
