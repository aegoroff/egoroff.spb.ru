# -*- coding: utf-8 -*-
from urlparse import urljoin
from lxml import etree
from apps.utils.paginator import Paginator, EmptyPage, InvalidPage
from flask import Blueprint, redirect, render_template, url_for
from apps.news.models import Post, Tag
import flask
import main
import site_map
from typographus import Typographus
import itertools
import util

mod = Blueprint(
    'news',
    __name__,
    template_folder='templates',
    url_prefix='/news'
)


def get_paginator(posts, page, posts_per_page=20):
    paginator = Paginator(posts, posts_per_page)
    try:
        posts = paginator.page(page)
    except (EmptyPage, InvalidPage):
        posts = paginator.page(paginator.num_pages)
    return posts


main_section_item = site_map.MAP[1]

POSTS_QUERY = "WHERE is_public = True ORDER BY created DESC"

TAG_RANK = ("tagRank10", "tagRank9",
            "tagRank8", "tagRank7", "tagRank6",
            "tagRank5", "tagRank4", "tagRank3",
            "tagRank2", "tagRank1")


def compute_rank(tag_title, count, totalcount):
    """Compute taglevel from frequency of tag"""
    index = int(float(count) / float(totalcount) * 10)
    return Tag(tag_title, TAG_RANK[index])


def create_tag_rank(articles):
    """Create tag rank"""
    tag_count_dict = {}
    total_count = 0
    for article in articles:
        for t in article.tags:
            total_count += 1
            if t in tag_count_dict:
                tag_count_dict[t] += 1
            else:
                tag_count_dict[t] = 1
    tags = []
    append = lambda tag_title: tags.append(compute_rank(tag_title, tag_count_dict[tag_title], total_count))
    map(append, tag_count_dict)
    return sorted(tags, key=lambda tag: tag.title)


class FileResolver(etree.Resolver):
    def resolve(self, url, identifier, context):
        return self.resolve_filename(url, context)

MONTHS = {
    1 : u"Январь",
    2 : u"Февраль",
    3 : u"Март",
    4 : u"Апрель",
    5 : u"Май",
    6 : u"Июнь",
    7 : u"Июль",
    8 : u"Август",
    9 : u"Сентябрь",
    10 : u"Октябрь",
    11 : u"Ноябрь",
    12 : u"Декабрь"
}

@mod.route('/', defaults={'page': 1})
@mod.route('/page/<int:page>/')
def index(page):
    posts = Post.gql(POSTS_QUERY)
    breadcrumbs = main.create_breadcrumbs([])
    title = main_section_item[site_map.TITLE]
    tag = util.param('tag')
    if tag:
        breadcrumbs = main.create_breadcrumbs([main_section_item])
        title = u"Все посты по метке: {0}".format(tag)
        query = "WHERE is_public = True AND tags IN (:1) ORDER BY created DESC"
        posts = Post.gql(query, tag)

    all_query = Post.gql("WHERE is_public = True  ORDER BY created DESC")
    all_posts = all_query.fetch()

    archieve = {}
    for ym, group in itertools.groupby(all_posts, key=lambda post: (post.created.year, post.created.month)):
        if ym[0] not in archieve:
            archieve[ym[0]] = []
        for m, months in itertools.groupby(group, key=lambda p: p.created.month):
            posts_count = len(filter(None, months))
            k = (m, posts_count)
            archieve[ym[0]].append(k)

    posts = get_paginator(posts, page)

    return render_template(
        'news/index.html',
        title=title,
        parent_id=main_section_item[site_map.ID],
        current_id=main_section_item[site_map.ID],
        posts=posts,
        archieve=archieve,
        months=MONTHS,
        tag_selected=tag,
        tags=create_tag_rank(all_posts),
        key=main_section_item[site_map.ID],
        breadcrumbs=breadcrumbs
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
        xslt_root = etree.parse('apache/apache_manualpage.xsl', parser)
        transform = etree.XSLT(xslt_root)

        content = unicode(transform(xml_input))

    #typo = Typographus()
    #if content:
    #    content = typo.typo_text(content)

    original_limit = 5
    limit = original_limit
    offset = 0
    while True:
        keys = Post.query(Post.is_public == True).order(-Post.created).fetch(limit, keys_only=True, offset=offset)
        found = False
        for k in keys:
            if k.id() == key_id:
                found = True
                break
        if not found:
            offset += limit
        else:
            break

    if offset > 0:
        limit += offset
    offset += original_limit

    posts = Post.gql("{0} LIMIT {1}".format(POSTS_QUERY, limit))
    last = posts.fetch()
    return render_template(
        'news/post.html',
        title=post.title,
        main_post=post,
        content=content,
        last=last,
        limit=original_limit,
        offset=offset,
        full_uri=urljoin(flask.request.url_root, flask.url_for('news.post', key_id=key_id)),
        breadcrumbs=main.create_breadcrumbs([main_section_item])
    )