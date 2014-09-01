# -*- coding: utf-8 -*-

from flask import Blueprint, render_template, redirect, url_for
from apps.utils.blobstore import send_blob
from auth import current_user_db, current_user_key, is_logged_in
from apps.file.models import File

mod = Blueprint(
    'file',
    __name__,
    url_prefix='/f',
    template_folder='templates'
)

def _check_owner(file):
    return file.is_public or (is_logged_in()
                                and  (current_user_db().admin
                                or current_user_key() == file.owner))

@mod.route('/<string:file_key>', methods=['GET'], endpoint='get')
def get_file(file_key):
    file = File.query(File.uid==file_key).get()
    if not file or not file.blob_key:
        return render_template('file/not_found.html')
    if file.title_filename:
        return redirect(url_for('file.get_w_name',
            file_key=file_key, name=file.title_filename))
    return send_blob(file.blob_key, content_type=file.content_type)

@mod.route('/<string:file_key>/<string:name>',
    methods=['GET'],
    endpoint='get_w_name')
def get_file_w_name(file_key, name):
    file = File.query(File.uid==file_key).get()
    if not file or not file.blob_key:
        return render_template('file/not_found.html')
    return send_blob(file.blob_key, content_type=file.content_type)

_blueprints = (mod,)