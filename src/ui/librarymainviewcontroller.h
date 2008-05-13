/*
 * niepce - ui/librarymainviewcontroller.h
 *
 * Copyright (C) 2007-2008 Hubert Figuiere
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


#ifndef __UI_LIBRARYMAINVIEWCONTROLLER_H__
#define __UI_LIBRARYMAINVIEWCONTROLLER_H__


#include <gtkmm/iconview.h>
#include <gtkmm/liststore.h>
#include <gtkmm/treestore.h>
#include <gtkmm/scrolledwindow.h>
#include <gtkmm/paned.h>

#include "librarymainview.h"
#include "db/libfile.h"
#include "libraryclient/libraryclient.h"
#include "framework/controller.h"
#include "framework/notification.h"
#include "metadatapanecontroller.h"
#include "selectioncontroller.h"
#include "darkroommodule.h"

namespace Gtk {
	class Widget;
}


namespace ui {

	class LibraryMainViewController
		: public framework::Controller,
		  public IImageSelectable
	{
	public:
		typedef boost::shared_ptr<LibraryMainViewController> Ptr;
		typedef boost::weak_ptr<LibraryMainViewController> WeakPtr;

		class LibraryListColumns 
			: public Gtk::TreeModelColumnRecord
		{
		public:
			
			LibraryListColumns()
				{ 
					add(m_pix);
					add(m_libfile);
				}
			Gtk::TreeModelColumn<Glib::RefPtr<Gdk::Pixbuf> > m_pix;
			Gtk::TreeModelColumn<db::LibFile::Ptr> m_libfile;
		};

		void on_lib_notification(const framework::Notification::Ptr &);
		void on_tnail_notification(const framework::Notification::Ptr &);

		/** called when somehing is selected by the shared selection */
		void on_selected(int id);
		void on_image_activated(int id);

		virtual Gtk::IconView * image_list();
		virtual int get_selected();
		virtual void select_image(int id);
	protected:
		virtual Gtk::Widget * buildWidget();
		virtual void on_ready();
	private:
		libraryclient::LibraryClient::Ptr getLibraryClient();

		// managed widgets...
		LibraryMainView              m_mainview;
		Gtk::IconView                m_librarylistview;
		Gtk::ScrolledWindow          m_scrollview;
		// library split view
		Gtk::HPaned                  m_lib_splitview;
		Gtk::ScrolledWindow          m_lib_metapanescroll;
		MetaDataPaneController::Ptr  m_metapanecontroller;

		DarkroomModule::Ptr          m_darkroom;

		LibraryListColumns           m_columns;
		Glib::RefPtr<Gtk::ListStore> m_model;
		std::map<int, Gtk::TreeIter> m_idmap;
	};

}

#endif
