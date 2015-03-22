# -*- coding: utf-8 -*-
from urlparse import urljoin
from google.appengine.api import memcache
from lxml import etree
from apps.utils.paginator import Paginator, EmptyPage, InvalidPage
from flask import Blueprint, render_template
from apps.news.models import Post, Tag
import flask
import config
from flask import abort
import itertools
import util
from werkzeug.contrib.atom import AtomFeed

mod = Blueprint(
    'news',
    __name__,
    template_folder='templates',
    url_prefix='/blog'
)

def get_paginator(posts, page, posts_per_page=20):
    paginator = Paginator(posts, posts_per_page)
    try:
        posts = paginator.page(page)
    except (EmptyPage, InvalidPage):
        posts = paginator.page(paginator.num_pages)
    return posts



TAG_RANK = ("tagRank10", "tagRank9",
            "tagRank8", "tagRank7", "tagRank6",
            "tagRank5", "tagRank4", "tagRank3",
            "tagRank2", "tagRank1")


def create_posts_query():
    """
    Creates new public posts query object with created sorting desc
    :return: new query instance
    """
    return Post.query(Post.is_public == True).order(-Post.created)


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
    1: u"Январь",
    2: u"Февраль",
    3: u"Март",
    4: u"Апрель",
    5: u"Май",
    6: u"Июнь",
    7: u"Июль",
    8: u"Август",
    9: u"Сентябрь",
    10: u"Октябрь",
    11: u"Ноябрь",
    12: u"Декабрь"
}


def month_tuple_to_string(month):
    ix = int(month[0])
    if ix > 12:
        name = u"За весь год"
    else:
        name = MONTHS[ix]
    return u'{0} ({1})'.format(name, month[1])

mod.add_app_template_filter(month_tuple_to_string, 'month_tuple_to_string')


@mod.route('/', defaults={'page': 1})
@mod.route('/page/<int:page>/')
def index(page):
    tag = util.param('tag')
    y = util.param('year')
    m = util.param('month')
    if tag or y or m:
        return abort(404)

    posts = create_posts_query()
    title = ''

    if page > 1:
        title = u" {0}-я страница".format(page)

    all_posts = util.run_query(create_posts_query())

    archieve = {}
    for year_and_month, group in itertools.groupby(all_posts, key=lambda post: (post.created.year, post.created.month)):
        if year_and_month[0] not in archieve:
            archieve[year_and_month[0]] = []
        for m, months in itertools.groupby(group, key=lambda p: p.created.month):
            posts_count = len(filter(None, months))
            item = (m, posts_count)
            archieve[year_and_month[0]].append(item)

    for y in archieve:
        total = 0
        for year_and_month in archieve[y]:
            total += year_and_month[1]
        item = (y, total)
        archieve[y].insert(0, item)

    posts = get_paginator(util.run_query(posts), page)

    return render_template(
        'news/index.html',
        title=title,
        posts=posts,
        archieve=archieve,
        tags=create_tag_rank(all_posts),
    )


@mod.route('/recent.atom')
def recent_feed():
    feed = AtomFeed('{0} feed'.format(config.CONFIG_DB.brand_name),
                    feed_url=flask.request.url, url=flask.request.url_root)
    limit = config.ATOM_FEED_LIMIT
    if util.param('limit'):
        limit = util.param('limit', int)

    articles = util.run_query(create_posts_query(), limit)

    make_external = lambda url: urljoin(flask.request.url_root, url)

    for article in articles:
        feed.add(article.title, unicode(article.short_text),
                 content_type='html',
                 author="Alexander Egorov",
                 url=make_external(flask.url_for('news.post', key_id=article.key.id())),
                 updated=article.modified,
                 published=article.created)
    return feed.get_response()


@mod.route('/<int:key_id>/', endpoint='post')
@mod.route('/<int:key_id>.html', endpoint='post')
def get_post(key_id):
    post = Post.retrieve_by_id(key_id)
    if not post or not post.is_public:
        abort(404)

    content = post.text
    if content and content.startswith('<?xml version="1.0"?>'):
        parser = etree.XMLParser(load_dtd=False, dtd_validation=False)
        parser.resolvers.add(FileResolver())

        xml_input = etree.fromstring(post.text, )
        xslt_root = etree.parse('apache/apache_manualpage.xsl', parser)
        transform = etree.XSLT(xslt_root)

        content = unicode(transform(xml_input))

    return render_template(
        'news/post.html',
        title=post.title,
        main_post=post,
        content=content
    )


def create_posts_keys(limit, offset):
    return 'posts_id_desc_limit_{0}_offset_{1}'.format(limit, offset)


def get_posts_ids(limit, offset):
    key = create_posts_keys(limit, offset)
    latest = memcache.get(key)
    if latest is None:
        keys = util.run_query(create_posts_query(), limit, offset=offset, keys_only=True)
        latest = ','.join([str(k.id()) for k in keys])
        memcache.add(key, latest, 86400)
    return [long(k) for k in latest.split(',')]