/*
 * niepce - db/library.h
 *
 * Copyright (C) 2007 Hubert Figuiere
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  
 * 02110-1301, USA
 */



#ifndef _DB_LIBRARY_H_
#define _DB_LIBRARY_H_

#include <string>

#include <boost/shared_ptr.hpp>

namespace db {

	class Library
	{
	public:
		typedef boost::shared_ptr<Library> Ptr;

		Library(const std::string & dir);
		virtual ~Library();

		/** set the main library directory */
		void setMainDir(const std::string & dir)
			{ m_maindir = dir; }
		/** return the main directory */
		const std::string & mainDir() const
			{ return m_maindir; }


	private:
		std::string m_maindir;
	};

}

#endif
