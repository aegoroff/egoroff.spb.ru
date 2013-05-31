# -*- coding: utf-8 -*-
from lxml import etree
from flask import Blueprint, redirect, render_template, url_for
from apps.news.models import Post
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

    posts = Post.query().order(-Post.created)
    last = posts.fetch(5)
    breadcrumbs =[i for i in main.breadcrumbs_home]
    breadcrumbs.append((main_section_item[site_map.ID], main_section_item[site_map.TITLE]))
    return render_template(
        'news/post.html',
        title=post.title,
        post=post,
        content=content,
        last=last,
        breadcrumbs=breadcrumbs
    )