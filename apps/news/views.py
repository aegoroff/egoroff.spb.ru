# -*- coding: utf-8 -*-
from urlparse import urljoin
from lxml import etree
from apps.utils.paginator import Paginator, EmptyPage, InvalidPage
from flask import Blueprint, redirect, render_template, url_for
from apps.news.models import Post
import flask
import main
import site_map
from typographus import Typographus


mod = Blueprint(
    'news',
    __name__,
    template_folder='templates',
    url_prefix='/news'
)


def get_paginator(posts, page, posts_per_page = 20):
    paginator = Paginator(posts, posts_per_page)
    try:
        posts = paginator.page(page)
    except (EmptyPage, InvalidPage):
        posts = paginator.page(paginator.num_pages)
    return posts

main_section_item = site_map.MAP[1]

POSTS_QUERY = "WHERE is_public = True ORDER BY created DESC"

class FileResolver(etree.Resolver):
    def resolve(self, url, identifier, context):
        return self.resolve_filename(url, context)

@mod.route('/', defaults={'page': 1})
@mod.route('/page/<int:page>/')
def index(page):
    posts = Post.gql(POSTS_QUERY)
    posts = get_paginator(posts, page)
    return render_template(
        'news/index.html',
        title=main_section_item[site_map.TITLE],
        parent_id=main_section_item[site_map.ID],
        current_id=main_section_item[site_map.ID],
        posts=posts,
        key=main_section_item[site_map.ID],
        breadcrumbs=main.create_breadcrumbs([])
    )

@mod.route('/rss/')
def rss():
    return main.recent_feed()

@mod.route('/<int:key_id>/', endpoint='post')
@mod.route('/<int:key_id>.html', endpoint='post')
def get_post(key_id):
    post = Post.retrieve_by_id(key_id)
    if not post:
        return redirect(url_for('news.index'))

    content = post.text
    if content and content.startswith('<?xml version="1.0"?>'):
        parser = etree.XMLParser(load_dtd=False, dtd_validation=False)
        parser.resolvers.add(FileResolver())

        xml_input = etree.fromstring(post.text, )
        stylesheet = main.apache_docs['rewriteguide'][0]
        xslt_root = etree.parse('apache/{0}.xsl'.format(stylesheet), parser)
        transform = etree.XSLT(xslt_root)

        content = unicode(transform(xml_input))

    #typo = Typographus()
    #if content:
    #    content = typo.typo_text(content)

    limit = 5
    posts = Post.gql("{0} LIMIT {1}".format(POSTS_QUERY, limit))
    last = posts.fetch()
    return render_template(
        'news/post.html',
        title=post.title,
        post=post,
        content=content,
        last=last,
        limit=limit,
        offset=limit,
        full_uri=urljoin(flask.request.url_root, flask.url_for('news.post', key_id=post.key.id())),
        breadcrumbs=main.create_breadcrumbs([main_section_item])
    )