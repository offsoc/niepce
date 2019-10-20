/*
 * niepce - lib.rs
 *
 * Copyright (C) 2017-2019 Hubert Figuière
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

extern crate chrono;
extern crate exempi;
extern crate gettextrs;
extern crate gio;
extern crate gio_sys;
extern crate glib;
extern crate glib_sys;
extern crate gtk;
extern crate gtk_sys;
extern crate libc;
extern crate multimap;
extern crate rusqlite;
#[macro_use]
extern crate try_opt;

// internal crates
#[macro_use]
extern crate npc_fwk;

pub mod engine;
pub mod libraryclient;
pub mod niepce;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::sync::Once;

/// Call this to initialize npc-fwk the gtk-rs bindings
#[no_mangle]
pub extern "C" fn niepce_init() {
    static START: Once = Once::new();

    START.call_once(|| {
        gtk::init().unwrap();
        npc_fwk::init();
    });
}
