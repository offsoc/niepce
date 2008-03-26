/*
 * niepce - ui/filmstripcontroller.h
 *
 * Copyright (C) 2008 Hubert Figuiere
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



#ifndef __UI_FILMSTRIPCONTROLLER_H_
#define __UI_FILMSTRIPCONTROLLER_H_

#include "framework/controller.h"
#include "framework/notificationcenter.h"
#include "ui/selectioncontroller.h"

namespace Gtk {
	class IconView;
}

namespace ui {


class FilmStripController
	: public framework::Controller,
	  public IImageSelectable
{
public:
	typedef boost::shared_ptr<FilmStripController> Ptr;
	typedef boost::weak_ptr<FilmStripController> WeakPtr;

	virtual Gtk::IconView * image_list();
	virtual int get_selected();
	virtual void select_image(int id);

	void on_tnail_notification(const framework::Notification::Ptr &);
	void on_lib_notification(const framework::Notification::Ptr &);
	
protected:
	virtual Gtk::Widget * buildWidget();
private:
	Gtk::IconView * m_thumbview;
};


}

#endif
