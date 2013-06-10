# -*- coding: utf-8 -*-
from urlparse import urljoin
import flask
from flask import Blueprint

__author__ = 'egorov'

mod = Blueprint(
    'compatibility',
    __name__
)


def redirect(key_id, remapping):
    if key_id in remapping:
        url = urljoin(flask.url_for('news.index'), '{0}.html'.format(remapping[key_id]))
        return flask.redirect(url, code=301)
    return flask.redirect(flask.url_for('news.index'), code=301)

# Redirection rules for the old site materials

@mod.route('/opinions/')
def opinions():
    return flask.redirect(flask.url_for('news.index'), code=301)

@mod.route('/news/<int:key_id>/', endpoint='post')
@mod.route('/news/<int:key_id>.html', endpoint='post')
def redirect_blog_post(key_id):
    path = '{0}.html'.format(key_id)
    url = urljoin(flask.url_for('news.index'), path)
    return flask.redirect(url, code=301)

@mod.route('/news/', defaults={'page': 1})
@mod.route('/news/page/<int:page>/')
def redirect_blog_index(page):
    path = ''
    if page > 1:
        path = 'page/{0}/'.format(page)
    home = flask.url_for('news.index')
    url = '{0}{1}'.format(home, path)
    return flask.redirect(url, code=301)

@mod.route('/news/rss/')
@mod.route('/blog/rss/')
@mod.route('/recent.atom')
def blog_rss():
    return flask.redirect(flask.url_for('news.recent_feed'))

@mod.route('/opinions/<int:key_id>.html')
def opinions_files(key_id):
    remapping = {
        1: 25002,
        4: 31001,
        8: 6003,
        11: 30001,
        13: 3006,
        18: 29001,
        21: 9002,
        22: 2004,
        24: 25003,
        25: 22002,
        26: 27002,
        27: 27001,
        28: 14004,
        29: 8003,
        30: 6004
    }
    return redirect(key_id, remapping)


@mod.route('/portfolio/<int:key_id>.html', methods=['GET'])
def portfolio_files(key_id):
    remapping = {
        1: 19002,
        3: 24001,
        5: 23001,
        6: 21002,
        7: 22001,
        8: 21001,
        9: 13004,
        10: 6007,
        11: 18001,
        12: 12004,
        13: 12003,
        14: 1006,
        15: 6005,
        16: 16002,
        17: 12002,
        18: 6006,
        19: 17001,
        21: 11004,
        22: 9004,
        23: 13003,
        24: 16001,
        25: 9003,
        26: 14005,
        27: 5004,
        28: 11003,
        29: 15001,
        30: 8002
    }
    return redirect(key_id, remapping)


@mod.route('/portfolio/download/', defaults={'doc': None})
@mod.route('/portfolio/apache/', defaults={'doc': None})
@mod.route('/portfolio/flickr/', defaults={'doc': None})
@mod.route('/apache/', defaults={'doc': None})
@mod.route('/portfolio/<doc>.html', methods=['GET'])
@mod.route('/apache/<doc>.html', methods=['GET'])
@mod.route('/portfolio/apache/<doc>.html', methods=['GET'])
def redirect_portfolio(doc):
    path = ''
    if doc:
        path = '{0}.html'.format(doc)
    url = urljoin(flask.url_for('portfolio.index'), path)
    return flask.redirect(url, code=301)