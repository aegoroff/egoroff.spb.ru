# -*- coding: utf-8 -*-
import os

from flask import Blueprint, redirect, render_template, url_for, request
import flask
from google.appengine.api import memcache
from google.appengine.ext import blobstore
from apps.news.admin import tags
from auth import admin_required
from apps.news.models import Post
from apps.news.admin.forms import PostForm

from datetime import datetime
from apps.file.models import Folder, File
from apps.utils.blobstore import get_uploads
import util

mod = Blueprint(
    'admin.news',
    __name__,
    url_prefix='/admin/news',
    template_folder='templates'
)

@mod.route('/')
@admin_required
def index():
    posts = Post.query().order(-Post.created)
    return render_template(
        'news/admin/posts.html',
        posts=posts,
        title=u'Блог'
    )

@mod.route('/new/', methods=['GET', 'POST'])
@admin_required
def new_post():
    form = PostForm(name=u'post')
    if form.validate_on_submit():
        post = Post()
        form.populate_obj(post)
        post.put()
        memcache.flush_all()
        return redirect(url_for('admin.news.index'))
    return render_template(
        'news/admin/post_new.html',
        form=form,
        title=u'Новый пост',
        allowed=tags.allowed(),
        image_upload_handler = blobstore.create_upload_url(flask.url_for('admin.news.add_blob_content'))
    )

@mod.route('/<int:key_id>/', methods=['GET', 'POST'])
@admin_required
def edit_post(key_id):
    post = Post.retrieve_by_id(key_id)
    if not post:
        return redirect(url_for('admin.news.index'))
    if request.method == 'POST' and 'delete_post' in request.form:
        post.key.delete()
        memcache.flush_all()
        return redirect(url_for('admin.news.index'))
    form = PostForm(obj=post)
    if form.validate_on_submit():
        form.populate_obj(post)
        post.put()
        memcache.flush_all()
        return redirect(url_for('admin.news.index'))
    return render_template(
        'news/admin/post_edit.html',
        form=form,
        post=post,
        title=post.title,
        allowed=tags.allowed(),
        image_upload_handler = blobstore.create_upload_url(flask.url_for('admin.news.add_blob_content'))
    )

@mod.route('/add_blob_content/', methods=['POST'])
@admin_required
def add_blob_content():

    response_object = {
      'status': 'success',
      'now': datetime.utcnow().isoformat(),
      'filelink': None,
    }

    folder = Folder.retrieve_by_id(3001)
    if not folder:
        response_object['status'] = 'failure'
        return util.jsonpify(response_object)
    upload_files = get_uploads(flask.request, 'file')
    if len(upload_files):
        blob_info = upload_files[0]
        if blob_info.size:
            f = File.create(
                blob_info.key(),
                size=blob_info.size,
                filename=os.path.basename(blob_info.filename.replace('\\','/')),
                content_type=blob_info.content_type)
            f.put()
            if f.get_cached_url():
                folder.files.append(f.key)
                folder.put()
                response_object['filelink'] = f.get_cached_url()
        else:
            blob_info.delete()
    return util.jsonpify(response_object)