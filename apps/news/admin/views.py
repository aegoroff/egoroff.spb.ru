# -*- coding: utf-8 -*-

from flask import Blueprint, redirect, render_template, url_for, request
from auth import admin_required
from apps.news.models import Post
from apps.news.admin.forms import PostForm

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
        posts=posts
    )

@mod.route('/new/', methods=['GET', 'POST'])
@admin_required
def new_post():
    form = PostForm()
    if form.validate_on_submit():
        post = Post()
        form.populate_obj(post)
        post.put()
        return redirect(url_for('admin.news.index'))
    return render_template(
        'news/admin/post_new.html',
        form=form
    )

@mod.route('/<int:key_id>/', methods=['GET', 'POST'])
@admin_required
def edit_post(key_id):
    post = Post.retrieve_by_id(key_id)
    if not post:
        return redirect(url_for('admin.news.index'))
    if request.method == 'POST' and 'delete_post' in request.form:
        post.key.delete()
        return redirect(url_for('admin.news.index'))
    form = PostForm(obj=post)
    if form.validate_on_submit():
        form.populate_obj(post)
        post.put()
        return redirect(url_for('admin.news.index'))
    return render_template(
        'news/admin/post_edit.html',
        form=form,
        post=post
    )