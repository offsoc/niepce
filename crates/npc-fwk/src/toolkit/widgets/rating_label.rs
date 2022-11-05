/*
 * niepce - crates/npc-fwk/src/toolkit/widgets/rating_label.rs
 *
 * Copyright (C) 2020-2022 Hubert Figuière
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

use std::cell::Cell;

use gdk4::prelude::*;
use glib::subclass::prelude::*;
use glib::subclass::Signal;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

struct Pixbufs {
    star: gdk4::Texture,
    unstar: gdk4::Texture,
}

lazy_static::lazy_static! {
    static ref PIXBUFS: Pixbufs = Pixbufs {
        star: gdk4::Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-set-star.png"),
        unstar: gdk4::Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-unset-star.png"),
    };
}

glib::wrapper! {
    pub struct RatingLabel(
        ObjectSubclass<RatingLabelPriv>)
        @extends gtk4::Widget;
}

pub struct RatingLabelPriv {
    editable: Cell<bool>,
    rating: Cell<i32>,
}

impl RatingLabelPriv {
    fn set_editable(&self, editable: bool) {
        self.editable.set(editable);
    }

    fn rating(&self) -> i32 {
        self.rating.get()
    }

    fn set_rating(&self, rating: i32) {
        self.rating.set(rating);
        let w = self.instance();
        w.queue_draw();
    }

    fn press_event(&self, _gesture: &gtk4::GestureClick, _: i32, x: f64, _: f64) {
        let new_rating = RatingLabel::rating_value_from_hit_x(x);
        if new_rating != self.rating.get() {
            self.set_rating(new_rating);
            self.instance()
                .emit_by_name::<()>("rating-changed", &[&new_rating]);
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for RatingLabelPriv {
    const NAME: &'static str = "RatingLabel";
    type Type = RatingLabel;
    type ParentType = gtk4::Widget;

    fn new() -> Self {
        Self {
            editable: Cell::new(true),
            rating: Cell::new(0),
        }
    }
}

impl ObjectImpl for RatingLabelPriv {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.instance();
        let click = gtk4::GestureClick::new();
        click.connect_pressed(glib::clone!(@weak obj => move |gesture, n, x, y| {
            obj.imp().press_event(gesture, n, x, y);
        }));
        obj.add_controller(&click);
    }

    fn signals() -> &'static [Signal] {
        use once_cell::sync::Lazy;
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder("rating-changed")
                .param_types([<i32>::static_type()])
                .run_last()
                .build()]
        });
        SIGNALS.as_ref()
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![glib::ParamSpecInt::new(
                "rating",
                "Rating",
                "The rating value",
                0,
                5,
                0,
                glib::ParamFlags::READWRITE,
            )]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "rating" => {
                let rating = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.set_rating(rating);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "rating" => self.rating.get().to_value(),
            _ => unimplemented!(),
        }
    }
}

pub trait RatingLabelExt {
    fn set_rating(&self, rating: i32);
    fn rating(&self) -> i32;
}

impl RatingLabelExt for RatingLabel {
    fn set_rating(&self, rating: i32) {
        self.imp().set_rating(rating);
    }

    fn rating(&self) -> i32 {
        self.imp().rating()
    }
}

impl RatingLabel {
    /// Connect to the signal `rating-changed`
    pub fn connect_rating_changed<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, i32) + 'static,
    {
        self.connect_local(
            "rating-changed",
            true,
            glib::clone!(@weak self as w => @default-return None, move |values| {
                if let Ok(rating) = values[0].get::<i32>() {
                    f(&w, rating);
                }
                None
            }),
        )
    }

    pub fn star() -> gdk4::Texture {
        PIXBUFS.star.clone()
    }

    pub fn unstar() -> gdk4::Texture {
        PIXBUFS.unstar.clone()
    }

    /// Return the geometry as (width, height)
    pub fn geometry() -> (f32, f32) {
        let star = Self::star();
        (star.width() as f32 * 5.0, star.height() as f32)
    }

    pub fn draw_rating(
        snapshot: &gtk4::Snapshot,
        rating: i32,
        star: &gdk4::Texture,
        unstar: &gdk4::Texture,
        x: f32,
        y: f32,
    ) {
        let rating = if rating == -1 { 0 } else { rating };

        let w = star.width() as f32;
        let h = star.height() as f32;
        let mut y = y;
        y -= h;
        snapshot.save();
        snapshot.translate(&graphene::Point::new(x, y));
        for i in 1..=5 {
            if i <= rating {
                star.snapshot(snapshot.upcast_ref::<gdk4::Snapshot>(), w as f64, h as f64);
            } else {
                unstar.snapshot(snapshot.upcast_ref::<gdk4::Snapshot>(), w as f64, h as f64);
            }
            snapshot.translate(&graphene::Point::new(w, 0.0));
        }
        snapshot.restore();
    }

    pub fn rating_value_from_hit_x(x: f64) -> i32 {
        let width: f64 = Self::star().width().into();
        (x / width).round() as i32
    }

    pub fn new(rating: i32, editable: bool) -> Self {
        let obj: Self = glib::Object::new(&[]);

        let priv_ = &obj.imp();
        priv_.set_editable(editable);
        priv_.set_rating(rating);
        obj
    }
}

impl WidgetImpl for RatingLabelPriv {
    fn measure(&self, orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
        let m = match orientation {
            gtk4::Orientation::Horizontal => RatingLabel::star().width() * 5,
            gtk4::Orientation::Vertical => RatingLabel::star().height(),
            _ => -1,
        };

        (m, m, -1, -1)
    }

    fn snapshot(&self, snapshot: &gtk4::Snapshot) {
        let star = RatingLabel::star();
        let x = 0_f32;
        let y = star.height() as f32;
        let widget = self.instance();
        let rating = (widget.downcast_ref::<RatingLabel>().unwrap())
            .imp()
            .rating
            .get(); // this shouldn't fail.
        RatingLabel::draw_rating(snapshot, rating, &star, &RatingLabel::unstar(), x, y);
    }
}
