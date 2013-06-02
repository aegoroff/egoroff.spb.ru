# -*- coding: utf-8 -*-
from urlparse import urljoin
from lxml import etree
import config
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

main_section_item = site_map.MAP[1]

class FileResolver(etree.Resolver):
    def resolve(self, url, identifier, context):
        return self.resolve_filename(url, context)

@mod.route('/')
def index():
    posts = Post.query(Post.is_public == True).order(-Post.created)
    posts = posts.fetch(config.ATOM_FEED_LIMIT, offset=0)
    return render_template(
        'news/index.html',
        title=main_section_item[site_map.TITLE],
        parent_id=main_section_item[site_map.ID],
        current_id=main_section_item[site_map.ID],
        posts=posts,
        key=main_section_item[site_map.ID],
        breadcrumbs=main.breadcrumbs_home
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

    posts = Post.query(Post.is_public == True).order(-Post.created)
    last = posts.fetch(5)
    breadcrumbs =[i for i in main.breadcrumbs_home]
    breadcrumbs.append((main_section_item[site_map.ID], main_section_item[site_map.TITLE]))
    return render_template(
        'news/post.html',
        title=post.title,
        post=post,
        content=content,
        last=last,
        full_uri=urljoin(flask.request.url_root, flask.url_for('news.post', key_id=post.key.id())),
        breadcrumbs=breadcrumbs
    )