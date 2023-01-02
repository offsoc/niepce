/*
 * niepce - niepce/ui/dialogs/import/mod.rs
 *
 * Copyright (C) 2008-2023 Hubert Figuière
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

mod camera_importer_ui;
mod directory_importer_ui;
mod importer_ui;
mod thumb_item;
mod thumb_item_row;

use camera_importer_ui::CameraImporterUI;
use directory_importer_ui::DirectoryImporterUI;
use importer_ui::{ImporterUI, SourceSelectedCallback};

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use glib::translate::*;
use gtk4::prelude::*;
use gtk_macros::get_widget;
use once_cell::sync::OnceCell;

use crate::ffi;
use crate::import::ImportRequest;
use npc_engine::importer::{ImportedFile, Importer};
use npc_fwk::toolkit::Thumbnail;
use npc_fwk::{dbg_out, err_out, on_err_out};
use thumb_item::ThumbItem;
use thumb_item_row::ThumbItemRow;

enum Event {
    /// Set Source `source` and `dest_dir`
    SetSource(String, String),
    /// The source changed. `id` in the combo box.
    SourceChanged(String),
    PreviewReceived(String, Thumbnail),
    AppendFiles(Vec<Box<dyn ImportedFile>>),
}

struct Widgets {
    dialog: gtk4::Dialog,
    import_source_combo: gtk4::ComboBoxText,
    importer_ui_stack: gtk4::Stack,
    destination_folder: gtk4::Entry,
    images_list_model: gio::ListStore,

    importers: HashMap<String, Rc<dyn ImporterUI>>,
    current_importer: RefCell<Option<Rc<dyn ImporterUI>>>,
}

impl Widgets {
    fn add_importer_ui(&mut self, importer: Rc<dyn ImporterUI>, tx: glib::Sender<Event>) {
        self.import_source_combo
            .append(Some(&importer.id()), importer.name());

        dbg_out!("setting up importer widget for {}", &importer.id());
        let importer_widget = importer.setup_widget(self.dialog.upcast_ref::<gtk4::Window>());
        self.importer_ui_stack
            .add_named(&importer_widget, Some(&importer.id()));
        importer.set_source_selected_callback(Box::new(move |source, dest_dir| {
            on_err_out!(tx.send(Event::SetSource(source.to_string(), dest_dir.to_string())));
        }));

        self.importers.insert(importer.id(), importer.clone());
    }

    fn clear_import_list(&self) {
        self.images_list_model.remove_all();
        self.destination_folder.set_text("");
    }

    fn importer_changed(&self, source: &str) {
        self.current_importer
            .replace(self.importers.get(source).cloned());
        self.importer_ui_stack.set_visible_child_name(source);
    }
}

#[derive(Default)]
struct State {
    source: String,
    dest_dir: PathBuf,
    // map images name to position in list store.
    images_list_map: HashMap<String, u32>,
}

pub struct ImportDialog {
    tx: glib::Sender<Event>,
    base_dest_dir: PathBuf,

    widgets: OnceCell<Widgets>,
    state: RefCell<State>,
}

impl ImportDialog {
    pub fn new() -> Rc<Self> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let app = npc_fwk::ffi::Application_app();
        let cfg = &app.config().cfg;
        let base_dest_dir = cfg
            .value_opt("base_import_dest_dir")
            .map(PathBuf::from)
            .or_else(|| glib::user_special_dir(glib::UserDirectory::Pictures))
            .unwrap_or_else(glib::home_dir);
        let dialog = Rc::new(ImportDialog {
            tx,
            base_dest_dir,
            widgets: OnceCell::new(),
            state: RefCell::new(State::default()),
        });

        rx.attach(
            None,
            glib::clone!(@strong dialog => move |e| {
                dialog.dispatch(e);
                glib::Continue(true)
            }),
        );

        dialog
    }

    fn dispatch(&self, e: Event) {
        match e {
            Event::SetSource(source, destdir) => self.set_source(&source, &destdir),
            Event::SourceChanged(source) => self.import_source_changed(&source),
            Event::PreviewReceived(path, thumbnail) => self.preview_received(&path, thumbnail),
            Event::AppendFiles(files) => self.append_files_to_import(&files),
        }
    }

    fn setup_widget(&self) -> &gtk4::Dialog {
        &self
            .widgets
            .get_or_init(|| {
                let builder = gtk4::Builder::from_resource("/org/gnome/Niepce/ui/importdialog.ui");
                get_widget!(builder, gtk4::Dialog, import_dialog);
                // get_widget!(builder, gtk4::ComboBox, date_tz_combo);
                get_widget!(builder, gtk4::Entry, destination_folder);
                get_widget!(builder, gtk4::Stack, importer_ui_stack);
                get_widget!(builder, gtk4::ComboBoxText, import_source_combo);

                get_widget!(builder, gtk4::ScrolledWindow, attributes_scrolled);
                let metadata_pane = ffi::metadata_pane_controller_new();
                let w = unsafe {
                    gtk4::Widget::from_glib_none(
                        metadata_pane.build_widget() as *mut gtk4_sys::GtkWidget
                    )
                };
                // add
                attributes_scrolled.set_child(Some(&w));

                get_widget!(builder, gtk4::ScrolledWindow, images_list_scrolled);
                let images_list_model = gio::ListStore::new(ThumbItem::static_type());
                let selection_model = gtk4::SingleSelection::new(Some(&images_list_model));
                let image_gridview = crate::ImageGridView::new(&selection_model, None, None);
                let factory = gtk4::SignalListItemFactory::new();
                image_gridview.set_factory(Some(&factory));
                factory.connect_setup(move |_, item| {
                    let child = ThumbItemRow::new();
                    item.set_child(Some(&child));
                });
                factory.connect_bind(move |_, item| {
                    if let Some(row) = item
                        .child()
                        .and_then(|row| row.downcast::<ThumbItemRow>().ok())
                    {
                        let thumb_item = item
                            .item()
                            .and_then(|item| item.downcast::<ThumbItem>().ok())
                            .unwrap();
                        if let Some(ref name) = thumb_item.name() {
                            row.set_label(name);
                        }
                        if let Some(pixbuf) = thumb_item.pixbuf() {
                            row.set_image(&pixbuf);
                        }
                    }
                });

                images_list_scrolled.set_child(Some(&*image_gridview));
                images_list_scrolled
                    .set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);

                let mut widgets = Widgets {
                    dialog: import_dialog,
                    import_source_combo: import_source_combo.clone(),
                    importer_ui_stack,
                    destination_folder,
                    images_list_model,
                    importers: HashMap::new(),
                    current_importer: RefCell::new(None),
                };

                let importer = DirectoryImporterUI::new();
                widgets.add_importer_ui(importer, self.tx.clone());
                let importer = CameraImporterUI::new();
                widgets.add_importer_ui(importer, self.tx.clone());

                import_source_combo.connect_changed(
                    glib::clone!(@strong self.tx as tx => move |combo| {
                        if let Some(source) = combo.active_id() {
                            on_err_out!(tx.send(Event::SourceChanged(source.to_string())));
                        }
                    }),
                );

                let app = npc_fwk::ffi::Application_app();
                let cfg = &app.config().cfg;
                let last_importer = cfg.value("last_importer", "DirectoryImporter");
                import_source_combo.set_active_id(Some(&last_importer));

                widgets
            })
            .dialog
    }

    pub fn run_modal<F>(&self, parent: Option<&gtk4::Window>, callback: F)
    where
        F: Fn(&gtk4::Dialog, gtk4::ResponseType) + 'static,
    {
        let dialog = self.setup_widget();
        dialog.set_transient_for(parent);
        dialog.set_default_response(gtk4::ResponseType::Close);
        dialog.set_modal(true);
        dialog.connect_response(callback);
        dialog.show();
    }

    pub fn import_request(&self) -> Option<ImportRequest> {
        self.widgets
            .get()?
            .current_importer
            .borrow()
            .as_ref()
            .map(|importer| ImportRequest::new(self.source(), self.dest_dir(), importer.importer()))
    }

    fn clear_import_list(&self) {
        if let Some(widgets) = self.widgets.get() {
            widgets.clear_import_list();
        }
        let mut state = self.state.borrow_mut();
        state.images_list_map.clear();
    }

    fn import_source_changed(&self, source: &str) {
        if let Some(widgets) = self.widgets.get() {
            widgets.importer_changed(source);
            self.state.borrow_mut().source = "".to_string();
            self.clear_import_list();
            let app = npc_fwk::ffi::Application_app();
            let cfg = &app.config().cfg;
            cfg.set_value("last_importer", source);
        }
    }

    fn importer(&self) -> Option<Rc<dyn Importer>> {
        self.widgets
            .get()?
            .current_importer
            .borrow()
            .as_ref()
            .map(|v| v.importer())
    }

    fn set_source(&self, source: &str, dest_dir: &str) {
        self.clear_import_list();

        if let Some(importer) = self.importer() {
            let tx = self.tx.clone();
            importer.list_source_content(
                source,
                Box::new(move |files| {
                    on_err_out!(tx.send(Event::AppendFiles(files)));
                }),
            );
        }

        let mut full_dest_dir = self.base_dest_dir.clone();
        full_dest_dir.push(dest_dir);
        let mut state = self.state.borrow_mut();
        state.source = source.to_string();
        state.dest_dir = full_dest_dir;

        if let Some(widgets) = self.widgets.get() {
            widgets.destination_folder.set_text(dest_dir);
        }
    }

    fn append_files_to_import(&self, files: &[Box<dyn ImportedFile>]) {
        let paths: Vec<String> = files
            .iter()
            .map(|f| {
                let path = f.path();
                dbg_out!("selected {}", &path);
                if let Some(widgets) = self.widgets.get() {
                    widgets
                        .images_list_model
                        .append(&ThumbItem::new(f.as_ref()));
                    self.state
                        .borrow_mut()
                        .images_list_map
                        .insert(path.to_string(), widgets.images_list_model.n_items() - 1);
                }
                path.to_string()
            })
            .collect();

        if let Some(importer) = self.importer() {
            let tx = self.tx.clone();
            importer.get_previews_for(
                &self.state.borrow().source,
                paths,
                Box::new(move |path, thumbnail| {
                    on_err_out!(tx.send(Event::PreviewReceived(path, thumbnail)));
                }),
            );
        }
    }

    fn preview_received(&self, path: &str, thumbnail: Thumbnail) {
        if let Some(idx) = self.state.borrow_mut().images_list_map.get(path) {
            self.widgets.get().and_then(|widgets| {
                widgets
                    .images_list_model
                    .item(*idx)
                    .and_then(|item| item.downcast::<ThumbItem>().ok())
                    .map(|item| {
                        item.set_pixbuf(thumbnail.make_pixbuf());
                        widgets.images_list_model.items_changed(*idx, 0, 0);

                        item
                    })
            });
        }
    }

    fn source(&self) -> String {
        self.state.borrow().source.clone()
    }

    fn dest_dir(&self) -> PathBuf {
        self.state.borrow().dest_dir.to_path_buf()
    }
}