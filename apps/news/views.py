# -*- coding: utf-8 -*-
from flask import Blueprint, redirect, render_template, url_for
from apps.news.models import Post
import main
import site_map


mod = Blueprint(
    'news',
    __name__,
    template_folder='templates',
    url_prefix='/news'
)

main_section_item = site_map.MAP[2]

@mod.route('/')
def index():
    posts = Post.query().order(-Post.created)
    return render_template(
        'news/index.html',
        title=main_section_item[site_map.TITLE],
        parent_id=main_section_item[site_map.ID],
        current_id=main_section_item[site_map.ID],
        posts=posts,
        key=main_section_item[site_map.ID],
        breadcrumbs=main.breadcrumbs_home
    )

@mod.route('/<int:key_id>/', endpoint='post')
def get_post(key_id):
    post = Post.retrieve_by_id(key_id)
    if not post:
        return redirect(url_for('news.index'))
    breadcrumbs =[i for i in main.breadcrumbs_home]
    breadcrumbs.append((main_section_item[site_map.ID], main_section_item[site_map.TITLE]))
    return render_template(
        'news/post.html',
        title=post.title,
        post=post,
        breadcrumbs=breadcrumbs
    )