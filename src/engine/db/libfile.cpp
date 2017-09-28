/*
 * niepce - db/libfile.cpp
 *
 * Copyright (C) 2007-2009 Hubert Figuiere
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

#include "libfile.hpp"
#include "engine/library/notification.hpp"
#include "fwk/base/debug.hpp"
#include "properties.hpp"

namespace eng {

// some glue for rust
QueriedContent::QueriedContent(library_id_t _container)
    : container(_container)
    , files(new LibFileList)
{
}

void QueriedContent::push(void* f)
{
    files->push_back(eng::libfile_wrap(static_cast<LibFile*>(f)));
}

LibFilePtr libfile_wrap(eng::LibFile *lf)
{
    return LibFilePtr(lf, &ffi::engine_db_libfile_delete);
}


LibFilePtr libfile_new(library_id_t id, library_id_t folder_id,
                       library_id_t fs_file_id, const char *path,
                       const char *name)
{
    return libfile_wrap(
        ffi::engine_db_libfile_new(id, folder_id, fs_file_id, path, name));
}

/**
 * Converts a mimetype, which is expensive to calculate, into a FileType.
 * @param mime The mimetype we want to know as a filetype
 * @return the filetype
 * @todo: add the JPEG+RAW file types.
 */
FileType mimetype_to_filetype(fwk::MimeType mime)
{
    if (mime.isDigicamRaw()) {
        return FileType::RAW;
    } else if (mime.isImage()) {
        return FileType::IMAGE;
    } else if (mime.isMovie()) {
        return FileType::VIDEO;
    } else {
        return FileType::UNKNOWN;
    }
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
