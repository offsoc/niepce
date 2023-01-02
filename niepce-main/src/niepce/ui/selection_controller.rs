/*
 * niepce - niepce/ui/selection_controller.rs
 *
 * Copyright (C) 2022 Hubert Figuière
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

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::rc::Rc;
use std::sync::Arc;

use gettextrs::gettext as i18n;
use gtk4::prelude::*;

use super::image_list_store::ImageListStoreWrap;
use super::ContentView;
use npc_engine::db;
use npc_engine::db::props::NiepceProperties as Np;
use npc_engine::db::{LibFile, NiepcePropertyIdx};
use npc_engine::library::notification::LibNotification;
use npc_engine::libraryclient::{ClientInterface, LibraryClient, LibraryClientHost};
use npc_engine::ThumbnailCache;
use npc_fwk::base::Signal;
use npc_fwk::toolkit::widgets::WrappedPropertyBag;
use npc_fwk::toolkit::{Controller, ControllerImpl, UndoCommand, UndoTransaction};
use npc_fwk::{dbg_out, err_out, PropertyValue};

#[derive(PartialEq)]
enum Direction {
    Forward,
    Backwards,
}

#[derive(Default)]
pub struct SelectionHandler {
    store: Box<ImageListStoreWrap>,
    content: Cell<ContentView>,
    pub signal_selected: Signal<db::LibraryId>,
    pub signal_activated: Signal<db::LibraryId>,
}

impl SelectionHandler {
    pub fn activated(&self, pos: u32) {
        let selection = self.store.get_file_id_at_pos(pos);
        if selection != 0 {
            self.signal_activated.emit(selection);
        }
    }

    pub fn selected(&self, pos: u32) {
        let selection = self.store.get_file_id_at_pos(pos);
        if selection != 0 {
            self.signal_selected.emit(selection);
        }
    }
}

pub struct SelectionController {
    imp_: RefCell<ControllerImpl>,
    client: Arc<LibraryClient>,
    pub handler: Rc<SelectionHandler>,
}

impl Controller for SelectionController {
    /// What to do when ready.
    fn on_ready(&self) {}

    /// Return the implementation
    fn imp(&self) -> Ref<'_, ControllerImpl> {
        self.imp_.borrow()
    }

    /// Return the mutable implementation
    fn imp_mut(&self) -> RefMut<'_, ControllerImpl> {
        self.imp_.borrow_mut()
    }
}

impl SelectionController {
    pub fn new(client_host: &LibraryClientHost) -> Rc<SelectionController> {
        let handler = Rc::new(SelectionHandler::default());
        handler.store.selection_model().connect_selection_changed(
            glib::clone!(@weak handler => move |model, _, _| {
                let pos = model.selected();
                handler.selected(pos);
            }),
        );

        Rc::new(SelectionController {
            imp_: RefCell::new(ControllerImpl::default()),
            client: client_host.client().client(),
            handler,
        })
    }

    pub fn on_lib_notification(&self, ln: &LibNotification, thumbnail_cache: &ThumbnailCache) {
        self.handler
            .store
            .on_lib_notification(ln, &self.client, thumbnail_cache);
    }

    pub fn list_store(&self) -> &ImageListStoreWrap {
        &self.handler.store
    }

    /// Get the file with `id`.
    pub fn file(&self, id: db::LibraryId) -> Option<LibFile> {
        self.handler.store.file(id)
    }

    pub fn selection(&self) -> Option<db::LibraryId> {
        let pos = self.handler.store.selection_model().selected();
        if pos == gtk4::INVALID_LIST_POSITION {
            None
        } else {
            Some(self.handler.store.get_file_id_at_pos(pos))
        }
    }

    pub fn select_previous(&self) {
        self.selection_move(Direction::Backwards)
    }

    pub fn select_next(&self) {
        self.selection_move(Direction::Forward)
    }

    fn selection_move(&self, direction: Direction) {
        let selection = self.selection();
        if selection.is_none() {
            return;
        }

        let pos = self.handler.store.pos_from_id(selection.unwrap());
        if pos.is_none() {
            return;
        }
        let mut pos = pos.unwrap();

        let moved = if direction == Direction::Backwards {
            if pos != 0 {
                pos -= 1;
                true
            } else {
                false
            }
        } else {
            pos += 1;
            (pos as usize) < self.handler.store.len()
        };

        if moved {
            self.handler.store.selection_model().set_selected(pos);
        }
    }

    /// Rotate the selection by angle (in degrees), clockwise.
    /// A negative value goes counter clockwise.
    pub fn rotate(&self, _angle: i32) {
        err_out!("rotate is not implemented");
    }

    fn set_one_metadata(
        &self,
        undo_label: &str,
        file_id: db::LibraryId,
        meta: NiepcePropertyIdx,
        old_value: i32,
        new_value: i32,
    ) -> bool {
        let client_undo = self.client.clone();
        let client_redo = self.client.clone();
        npc_fwk::toolkit::undo_do_command(
            undo_label,
            Box::new(move || {
                client_redo.set_metadata(file_id, Np::Index(meta), &PropertyValue::Int(new_value));
                npc_fwk::toolkit::Storage::Void
            }),
            Box::new(move |_| {
                client_undo.set_metadata(file_id, Np::Index(meta), &PropertyValue::Int(old_value));
            }),
        );
        true
    }

    fn set_metadata(
        &self,
        undo_label: &str,
        file_id: db::LibraryId,
        props: &WrappedPropertyBag,
        old: &WrappedPropertyBag,
    ) -> bool {
        let mut undo = Box::new(UndoTransaction::new(undo_label));
        for key in props.0.keys() {
            let old_value = old.0.get(key).cloned().unwrap_or(PropertyValue::Empty);
            let new_value = props.0.get(key).cloned().unwrap();
            let key = *key;
            let client_undo = self.client.clone();
            let client_redo = self.client.clone();
            let command = UndoCommand::new(
                Box::new(move || {
                    client_redo.set_metadata(file_id, Np::from(key), &new_value);
                    npc_fwk::toolkit::Storage::Void
                }),
                Box::new(move |_| {
                    client_undo.set_metadata(file_id, Np::from(key), &old_value);
                }),
            );
            undo.add(command);
        }
        undo.execute();
        npc_fwk::ffi::Application_app().begin_undo(undo);
        true
    }

    pub fn set_label(&self, label: i32) {
        self.set_property(db::NiepcePropertyIdx::NpXmpLabelProp, label);
    }

    /// Set rating of selection
    pub fn set_rating(&self, rating: i32) {
        self.set_property(db::NiepcePropertyIdx::NpXmpRatingProp, rating);
    }

    /// Set rating of specific file.
    pub fn set_rating_of(&self, id: db::LibraryId, rating: i32) {
        self.set_property_of(id, db::NiepcePropertyIdx::NpXmpRatingProp, rating);
    }

    pub fn set_flag(&self, flag: i32) {
        self.set_property(db::NiepcePropertyIdx::NpNiepceFlagProp, flag);
    }

    fn set_property(&self, idx: db::NiepcePropertyIdx, value: i32) {
        dbg_out!("property {} = {}", idx.repr, value);
        if let Some(selection) = self.selection() {
            self.set_property_of(selection, idx, value)
        }
    }

    fn set_property_of(&self, id: db::LibraryId, idx: db::NiepcePropertyIdx, value: i32) {
        if let Some(mut file) = self.handler.store.file(id) {
            dbg_out!("old property is {}", file.property(Np::Index(idx)));
            let old_value = file.property(Np::Index(idx));
            let action = match idx {
                NiepcePropertyIdx::NpNiepceFlagProp => i18n("Set Flag"),
                NiepcePropertyIdx::NpXmpRatingProp => i18n("Set Rating"),
                NiepcePropertyIdx::NpXmpLabelProp => i18n("Set Label"),
                _ => i18n("Set Property"),
            };
            self.set_one_metadata(&action, id, idx, old_value, value);
            // we need to set the property here so that undo/redo works
            // consistently.
            file.set_property(Np::Index(idx), value);
        } else {
            err_out!("requested file {} not found!", id);
        }
    }

    pub fn set_properties(&self, props: &WrappedPropertyBag, old: &WrappedPropertyBag) {
        if let Some(selection) = self.selection() {
            self.set_metadata(&i18n("Set Properties"), selection, props, old);
        }
    }

    pub fn content_will_change(&self, content: super::ContentView) {
        self.handler.store.clear_content();
        self.handler.content.set(content);
    }

    pub fn write_metadata(&self) {
        if let Some(selection) = self.selection() {
            self.client.write_metadata(selection);
        }
    }

    /// Delete the selecton fron the view.
    /// What delete means depend on the view. In an album it removes from the album
    /// From a folder it moves to trash.
    pub fn delete_from_view(&self) {
        if let Some(selection) = self.selection() {
            if let Some(ref f) = self.handler.store.file(selection) {
                match self.handler.content.get() {
                    ContentView::Album(id) => {
                        self.remove_from_album(id, f);
                    }
                    ContentView::Folder(_) => {
                        self.move_file_to_trash(f);
                    }
                    // XXX handle remove from keyword.
                    _ => {}
                }
            }
        }
    }

    /// Remove file `f` from `album`
    fn remove_from_album(&self, album: db::LibraryId, f: &LibFile) {
        let file_id = f.id();
        let client_undo = self.client.clone();
        let client_redo = self.client.clone();
        npc_fwk::toolkit::undo_do_command(
            &i18n("Remove from album"),
            Box::new(move || {
                client_redo.remove_from_album(&[file_id], album);
                npc_fwk::toolkit::Storage::Void
            }),
            Box::new(move |_| client_undo.add_to_album(&[file_id], album)),
        );
    }

    /// Move the file `f` to the trash.
    fn move_file_to_trash(&self, f: &LibFile) {
        let trash_folder = self.client.get_trash_id();
        let file_id = f.id();
        let from_folder = f.folder_id();
        let client_undo = self.client.clone();
        let client_redo = self.client.clone();
        npc_fwk::toolkit::undo_do_command(
            &i18n("Move to Trash"),
            Box::new(move || {
                client_redo.move_file_to_folder(file_id, from_folder, trash_folder);
                npc_fwk::toolkit::Storage::Void
            }),
            Box::new(move |_| client_undo.move_file_to_folder(file_id, trash_folder, from_folder)),
        );
    }

    /// Move selection to trash
    pub fn move_to_trash(&self) {
        if let Some(selection) = self.selection() {
            if let Some(ref f) = self.handler.store.file(selection) {
                self.move_file_to_trash(f);
            }
        }
    }
}