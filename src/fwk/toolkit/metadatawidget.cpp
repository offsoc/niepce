/*
 * niepce - fwk/toolkit/metadatawidget.cpp
 *
 * Copyright (C) 2008-2020 Hubert Figuière
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


#include <utility>

#include <boost/lexical_cast.hpp>
#include <boost/format.hpp>
#include <boost/rational.hpp>

#include <glibmm/i18n.h>
#include <gtkmm/label.h>
#include <gtkmm/entry.h>
#include <gtkmm/textview.h>
#include <gtkmm/drawingarea.h>

#include "fwk/base/debug.hpp"
#include "fwk/base/autoflag.hpp"
#include "fwk/utils/exempi.hpp"
#include "fwk/utils/stringutils.hpp"
#include "fwk/toolkit/widgets/notabtextview.hpp"
#include "fwk/toolkit/widgets/tokentextview.hpp"

#include "rust_bindings.hpp"

#include "metadatawidget.hpp"

namespace fwk {

MetaDataWidget::MetaDataWidget(const Glib::ustring & title)
    : ToolboxItemWidget(title),
      m_fmt(nullptr),
      m_update(false)
{
    m_table.set_column_homogeneous(true);
    m_table.set_row_homogeneous(false);
    m_table.insert_column(0);
    m_table.insert_column(0);
    m_table.set_margin_start(8);
    add(m_table);
    set_sensitive(false);
}

void MetaDataWidget::set_data_format(const MetaDataSectionFormat * fmt)
{
    m_fmt = fmt;
    create_widgets_for_format(fmt);
}

void MetaDataWidget::rating_callback(GtkWidget* w, gint rating, gpointer user_data)
{
    auto self = static_cast<MetaDataWidget*>(user_data);
    auto id = GPOINTER_TO_INT(g_object_get_data(G_OBJECT(w), "id"));
    self->on_int_changed(rating, id);
}

Gtk::Widget*
MetaDataWidget::create_star_rating_widget(bool readonly, uint32_t id)
{
    Gtk::DrawingArea* r =
        Gtk::manage(Glib::wrap(
                        GTK_DRAWING_AREA(ffi::fwk_rating_label_new(0, !readonly))));
    if (!readonly) {
        r->set_data("id", GINT_TO_POINTER(id));
        g_signal_connect(r->gobj(), "rating-changed", G_CALLBACK(rating_callback), this);
    }
    return r;
}

Gtk::Widget*
MetaDataWidget::create_string_array_widget(bool readonly, uint32_t id)
{
    fwk::TokenTextView * ttv = Gtk::manage(new fwk::TokenTextView());
    if(!readonly) {
        ttv->signal_focus_out_event().connect(
            sigc::bind(
                sigc::mem_fun(*this, 
                              &MetaDataWidget::on_string_array_changed),
                ttv, id));
    }
    return ttv;
}

Gtk::Widget*
MetaDataWidget::create_text_widget(bool readonly, uint32_t id)
{
    if(readonly) {
        Gtk::Label * l = Gtk::manage(new Gtk::Label());
        l->set_xalign(0.0f);
        l->set_yalign(0.5f);
        // This will prevent the label from being expanded.
        l->set_ellipsize(Pango::ELLIPSIZE_MIDDLE);
        return l;
    }

    Gtk::TextView * e = Gtk::manage(new NoTabTextView());
    e->set_editable(true);
    e->set_wrap_mode(Gtk::WRAP_WORD);
    e->signal_focus_out_event().connect(
        sigc::bind(
            sigc::mem_fun(*this,
                          &MetaDataWidget::on_text_changed),
            e->get_buffer(), id));
    return e;
}

Gtk::Widget*
MetaDataWidget::create_string_widget(bool readonly, uint32_t id)
{
    if(readonly) {
        Gtk::Label * l = Gtk::manage(new Gtk::Label());
        l->set_xalign(0.0f);
        l->set_yalign(0.5f);
        // This will prevent the label from being expanded.
        l->set_ellipsize(Pango::ELLIPSIZE_MIDDLE);
        return l;
    }

    Gtk::Entry * e = Gtk::manage(new Gtk::Entry());
    e->set_has_frame(false); // TODO make that a custom widget
    e->signal_focus_out_event().connect(
        sigc::bind(
            sigc::mem_fun(*this,
                          &MetaDataWidget::on_str_changed),
            e, id));
    return e;
}

Gtk::Widget*
MetaDataWidget::create_date_widget(bool /*readonly*/, uint32_t id)
{
    // for now a date widget is just like a string. Read only
    return create_string_widget(true, id);
}

void
MetaDataWidget::create_widgets_for_format(const MetaDataSectionFormat * fmt)
{
    Gtk::Widget *w = nullptr;
    const MetaDataFormat * current = fmt->formats;
    int n_row = 0;

    while(current && current->label) {
        Gtk::Label *labelw = Gtk::manage(new Gtk::Label(
                                             Glib::ustring("<b>")
                                             + current->label + "</b>"));
        if(current->type != MetaDT::STRING_ARRAY
           && current->type != MetaDT::TEXT) {
            labelw->set_xalign(0.0f);
            labelw->set_yalign(0.5f);
        }
        else {
            // Text can wrap. Different alignment for the label
            labelw->set_xalign(0.0f);
            labelw->set_yalign(0.0f);
        }
        labelw->set_use_markup(true);

        switch(current->type) {
        case MetaDT::STAR_RATING:
            w = create_star_rating_widget(current->readonly, current->id);
            break;
        case MetaDT::STRING_ARRAY:
            w = create_string_array_widget(current->readonly, current->id);
            break;
        case MetaDT::TEXT:
            w = create_text_widget(current->readonly, current->id);
            break;
        case MetaDT::DATE:
            w = create_date_widget(current->readonly, current->id);
            break;
        default:
            w = create_string_widget(current->readonly, current->id);
            break;
        }

        m_table.insert_row(n_row + 1);
        m_table.attach(*labelw, 0, n_row, 1, 1);
        m_table.attach_next_to(*w, *labelw, Gtk::POS_RIGHT, 1, 1);
        m_data_map.insert(std::make_pair(current->id, w));

        current++;
        n_row++;
    }
    m_table.show_all();
}

void MetaDataWidget::clear_widget(const std::pair<const PropertyIndex, Gtk::Widget *> & p)
{
    AutoFlag flag(m_update);
    Gtk::Label * l = dynamic_cast<Gtk::Label*>(p.second);
    if(l) {
        l->set_text("");
        return;
    }
    Gtk::Entry * e = dynamic_cast<Gtk::Entry*>(p.second);
    if(e) {
        e->set_text("");
        return;
    }
    fwk::TokenTextView *ttv = dynamic_cast<fwk::TokenTextView*>(p.second);
    if(ttv) {
        ttv->set_tokens(fwk::TokenTextView::Tokens());
        return;
    }
    Gtk::TextView * tv = dynamic_cast<Gtk::TextView*>(p.second);
    if(tv) {
        tv->get_buffer()->set_text("");
        return;
    }
    Gtk::DrawingArea* rl = dynamic_cast<Gtk::DrawingArea*>(p.second);
    if (rl) {
        ffi::fwk_rating_label_set_rating(rl->gobj(), 0);
        return;
    }
}

void MetaDataWidget::set_data_source(const fwk::PropertyBagPtr& properties)
{
    DBG_OUT("set data source");
    m_current_data = properties;
    if(!m_data_map.empty()) {
        std::for_each(m_data_map.cbegin(), m_data_map.cend(),
                      [this] (const decltype(m_data_map)::value_type& p) {
                          this->clear_widget(p);
                      });
    }
    bool is_empty =
        static_cast<bool>(properties) ? fwk_property_bag_is_empty(properties.get()) : true;
    set_sensitive(!is_empty);
    if(is_empty) {
        return;
    }
    if(!m_fmt) {
        DBG_OUT("empty format");
        return;
    }

    const MetaDataFormat * current = m_fmt->formats;
    while(current && current->label) {
        auto result = get_value_for_property(*properties, current->id);
        if (!result.empty() || !current->readonly) {
            add_data(current, std::move(result));
        }
        else {
            DBG_OUT("get_property failed id = %d, label = %s",
                    current->id, current->label);
        }
        current++;
    }
}

bool MetaDataWidget::set_fraction_dec_data(Gtk::Widget* w,
                                           const PropertyValuePtr& value)
{
    if (!fwk_property_value_is_string(value.get())) {
        ERR_OUT("Data not string(fraction)");
        return false;
    }
    try {
        const std::string str_value = fwk::property_value_get_string(*value);
        DBG_OUT("set fraction dec %s", str_value.c_str());
        std::string frac = str(boost::format("%.1f")
                               % ffi::fwk_fraction_to_decimal(str_value.c_str()));
        AutoFlag flag(m_update);
        static_cast<Gtk::Label*>(w)->set_text(frac);
    }
    catch(...) {
        return false;
    }
    return true;
}

bool MetaDataWidget::set_fraction_data(Gtk::Widget* w,
                                       const PropertyValuePtr& value)
{
    if (!fwk_property_value_is_string(value.get())) {
        ERR_OUT("Data not string(fraction)");
        return false;
    }
    try {
        const std::string str_value = fwk::property_value_get_string(*value);
        DBG_OUT("set fraction %s", str_value.c_str());
        boost::rational<int> r
            = boost::lexical_cast<boost::rational<int>>(str_value);

        std::string frac = str(boost::format("%1%/%2%")
                               % r.numerator() % r.denominator());
        AutoFlag flag(m_update);
        static_cast<Gtk::Label*>(w)->set_text(frac);
    }
    catch(...) {
        return false;
    }
    return true;
}

bool MetaDataWidget::set_star_rating_data(Gtk::Widget* w,
                                          const PropertyValuePtr& value)
{
    if (!fwk_property_value_is_integer(value.get())) {
        ERR_OUT("Data not integer");
        return false;
    }
    try {
        int rating = fwk_property_value_get_integer(value.get());
        AutoFlag flag(m_update);
        ffi::fwk_rating_label_set_rating(static_cast<Gtk::DrawingArea*>(w)->gobj(), rating);
    }
    catch(...) {
        return false;
    }
    return true;
}

bool MetaDataWidget::set_string_array_data(Gtk::Widget* w, bool readonly,
                                           const PropertyValuePtr& value)
{
    try {
        AutoFlag flag(m_update);
        std::vector<std::string> tokens = fwk::property_value_get_string_array(*value);

        static_cast<fwk::TokenTextView*>(w)->set_tokens(tokens);
        static_cast<fwk::TokenTextView*>(w)->set_editable(!readonly);
    }
    catch(...) {
        return false;
    }
    return true;
}

bool MetaDataWidget::set_text_data(Gtk::Widget* w, bool readonly,
                                   const PropertyValuePtr& value)
{
    if (!fwk_property_value_is_string(value.get())) {
        ERR_OUT("Data not string.");
        return false;
    }
    try {
        AutoFlag flag(m_update);
        if(readonly) {
            static_cast<Gtk::Label*>(w)->set_text(
                fwk::property_value_get_string(*value));
        } else {
            static_cast<Gtk::TextView*>(w)->get_buffer()->set_text(
                fwk::property_value_get_string(*value));
        }
    }
    catch(...) {
        return false;
    }
    return true;
}

bool MetaDataWidget::set_string_data(Gtk::Widget* w, bool readonly,
                                     const PropertyValuePtr& value)
{
    if (!fwk_property_value_is_string(value.get())) {
        ERR_OUT("Data not string.");
        return false;
    }
    try {
        AutoFlag flag(m_update);
        if(readonly) {
            static_cast<Gtk::Label*>(w)->set_text(
                fwk::property_value_get_string(*value));
        } else {
            static_cast<Gtk::Entry*>(w)->set_text(
                fwk::property_value_get_string(*value));
        }
    }
    catch(...) {
        return false;
    }
    return true;
}

bool MetaDataWidget::set_date_data(Gtk::Widget* w, const PropertyValuePtr& value)
{
    if (!fwk_property_value_is_date(value.get())) {
        return false;
    }
    try {
        AutoFlag flag(m_update);
        const fwk::Date* date = fwk_property_value_get_date(value.get());
        if (date) {
            static_cast<Gtk::Label*>(w)->set_text(fwk::date_to_string(date));

            DBG_OUT("setting date data %s", fwk::date_to_string(date).c_str());
        } else {
            return false;
        }
    }
    catch(...) {
        return false;
    }
    return true;
}

void MetaDataWidget::add_data(const MetaDataFormat * current,
                              fwk::Option<PropertyValuePtr>&& optional_value)
{
    if (optional_value.empty()) {
        return;
    }
    auto value = optional_value.unwrap();
    if (fwk_property_value_is_empty(value.get())) {
        return;
    }

    Gtk::Widget *w = nullptr;
    auto iter = m_data_map.find(current->id);
    if(iter == m_data_map.end()) {
        ERR_OUT("no widget for property");
        return;
    }

    w = static_cast<Gtk::Label*>(iter->second);

    switch(current->type) {
    case MetaDT::FRAC_DEC:
        set_fraction_dec_data(w, value);
        break;
    case MetaDT::FRAC:
        set_fraction_data(w, value);
        break;
    case MetaDT::STAR_RATING:
        set_star_rating_data(w, value);
        break;
    case MetaDT::STRING_ARRAY:
        set_string_array_data(w, current->readonly, value);
        break;
    case MetaDT::TEXT:
        set_text_data(w, current->readonly, value);
        break;
    case MetaDT::DATE:
        set_date_data(w, value);
        break;
    default:
        if (!set_string_data(w, current->readonly, value)) {
            ERR_OUT("failed to set value for %u", current->id);
        }
        break;
    }
}

bool MetaDataWidget::on_str_changed(GdkEventFocus*, Gtk::Entry *e,
                                    fwk::PropertyIndex prop)
{
    if(m_update) {
        return true;
    }
    emit_metadata_changed(prop, fwk::property_value_new(e->get_text()));
    return true;
}

bool MetaDataWidget::on_text_changed(GdkEventFocus*,
                                     Glib::RefPtr<Gtk::TextBuffer> b,
                                     fwk::PropertyIndex prop)
{
    if(m_update) {
        return true;
    }
    emit_metadata_changed(prop,
                          fwk::property_value_new(b->get_text()));
    return true;
}

bool MetaDataWidget::on_string_array_changed(GdkEventFocus*,
                                             fwk::TokenTextView * ttv,
                                             fwk::PropertyIndex prop)
{
    if(m_update) {
        return true;
    }
    fwk::TokenTextView::Tokens tok;
    ttv->get_tokens(tok);
    emit_metadata_changed(prop,
                          fwk::property_value_new(tok));
    return true;
}

void MetaDataWidget::on_int_changed(int value, fwk::PropertyIndex prop)
{
    if(m_update) {
        return;
    }
    emit_metadata_changed(prop, fwk::property_value_new(value));
}

void MetaDataWidget::emit_metadata_changed(fwk::PropertyIndex prop,
                                           const fwk::PropertyValuePtr & value)
{
    fwk::PropertyBagPtr props = fwk::property_bag_new();
    fwk::PropertyBagPtr old_props = fwk::property_bag_new();
    fwk::set_value_for_property(*props, prop, *value);
    auto result = fwk::get_value_for_property(*m_current_data, prop);
    if (!result.empty()) {
        fwk::set_value_for_property(*old_props, prop, *result.unwrap());
    }
    signal_metadata_changed.emit(props, old_props);
}

}

/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:80
  End:
*/
