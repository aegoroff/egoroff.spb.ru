# -*- coding: utf-8 -*-
from google.appengine.ext import ndb
from apps.news.views import POSTS_QUERY
import config

from flask import Blueprint, render_template, current_app, request
import json
from util import param, jsonify_model_db
from apps.api.utils import except_wrap, ApiException
import datetime
from apps.news.models import Post
import util

mod = Blueprint(
    'api.v2',
    __name__,
    url_prefix='/api/v2',
    template_folder='templates'
)

def render_json(value):
    return current_app.response_class(json.dumps(value,
            indent=None if request.is_xhr else 2), mimetype='application/json')

@mod.route('/')
def index():
    return render_template(
        'api/v2/index.html'
    )

@mod.route('/posts.json')
@except_wrap
def products_json():
    year = util.param('year', int)
    month = util.param('month', int)
    tag = util.param('tag')
    if year and month:
        current_month = datetime.datetime(year, month, 1)
        next_month = util.add_months(datetime.datetime(year, month, 1), 1)

        posts = Post.query(Post.is_public == True, ndb.AND(Post.created >= current_month,
                         Post.created < next_month)).order(-Post.created)
        q = posts.fetch()
    else:
        offset = util.param('offset', int) or 0
        limit = util.param('limit', int) or config.ATOM_FEED_LIMIT

        if tag:
            query = "WHERE is_public = True AND tags IN (:1) ORDER BY created DESC LIMIT {0} OFFSET {1}"
            articles = Post.gql(query.format(limit, offset), tag)
        else:
            articles = Post.gql("{0} LIMIT {1} OFFSET {2}".format(POSTS_QUERY, limit, offset))
        q = articles.fetch()
    return util.jsonify_model_dbs(q)

@mod.route('/post.json')
@except_wrap
def product_json():
    key_id = param('id', int)
    if not key_id:
        raise ApiException('Invalid request: "id" parameter not found.')

    product = None
    if key_id:
        product = Post.retrieve_by_id(key_id)
    if not product:
        if key_id:
            raise ApiException('Post with "%s" == %s not found' % ('id', key_id), status=404)
    return jsonify_model_db(product)