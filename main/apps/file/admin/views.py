# -*- coding: utf-8 -*-

from google.appengine.ext import blobstore
import flask
import os
from auth import admin_required
from apps.file.models import Folder, File
from apps.file.admin.forms import FolderForm, FileForm
from apps.utils.blobstore import get_uploads



mod = flask.Blueprint(
    'admin.file',
    __name__,
    url_prefix='/admin/file',
    template_folder='templates'
)

@mod.route('/')
@admin_required
def index():
    folders = Folder.query()
    return flask.render_template(
        'file/admin/folders.html',
        folders=folders,
        title=u'Файлы и папки'
    )

@mod.route('/add/', methods=['GET', 'POST'])
@admin_required
def add_folder():
    folders = Folder.query()
    form = FolderForm()
    if form.validate_on_submit():
        folder = Folder()
        form.populate_obj(folder)
        folder.put()
        return flask.redirect(flask.url_for('admin.file.index'))
    return flask.render_template(
        'file/admin/add_folder.html',
        form=form,
        folders=folders,
        title=u'Добавить папку'
    )

@mod.route('/<int:key_id>/', methods=['GET', 'POST'])
@admin_required
def edit_folder(key_id):
    folder = Folder.retrieve_by_id(key_id)
    if not folder:
        return flask.redirect(flask.url_for('admin.file.index'))
    if flask.request.method == 'POST' and 'delete_folder' in flask.request.form:
        folder.key.delete()
        return flask.redirect(flask.url_for('admin.file.index'))
    form = FolderForm(obj=folder)
    if form.validate_on_submit():
        form.populate_obj(folder)
        folder.put()
        return flask.redirect(flask.url_for('admin.file.index'))
    file_form = FileForm()
    add_url = blobstore.create_upload_url(flask.url_for('admin.file.add_file', key_id=folder.key.id()))
    return flask.render_template(
        'file/admin/edit_folder.html',
        form=form,
        folder=folder,
        active_folder=folder.key,
        file_form=file_form,
        add_url=add_url,
        title='Edit'
    )


@mod.route('/<int:key_id>/add_file/', methods=['POST'])
@admin_required
def add_file(key_id):
    folder = Folder.retrieve_by_id(key_id)
    if not folder:
        return flask.redirect(flask.url_for('admin.file.index'))
    upload_files = get_uploads(flask.request, 'file')
    form = FileForm()
    if len(upload_files):
        blob_info = upload_files[0]
        if blob_info.size:
            f = File.create(
                blob_info.key(),
                size=blob_info.size,
                filename=os.path.basename(blob_info.filename.replace('\\','/')),
                content_type=blob_info.content_type)
            form.populate_obj(f)
            f.put()
            if f.get_cached_url():
                folder.files.append(f.key)
                folder.put()
        else:
            blob_info.delete()
    return flask.redirect(flask.url_for('admin.file.edit_folder', key_id=key_id))

@mod.route('/<int:key_id>/del_file/<int:file_key>/', methods=['POST'])
@admin_required
def del_file(key_id, file_key):
    folder = Folder.retrieve_by_id(key_id)
    if not folder:
        return flask.redirect(flask.url_for('admin.file.index'))
    for i, f in enumerate(folder.files):
        if f.id() == file_key:
            f.delete()
            del folder.files[i]
            break
    else:
        return flask.redirect(flask.url_for('admin.file.edit_folder', key_id=key_id))
    folder.put()
    return flask.redirect(flask.url_for('admin.file.edit_folder', key_id=key_id))

