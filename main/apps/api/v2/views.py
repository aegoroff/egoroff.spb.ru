# -*- coding: utf-8 -*-
import random
from apps.news.views import get_posts_ids, create_posts_query
import config

from flask import Blueprint, render_template, current_app, request
import json
from util import param, jsonify_model_db
from apps.api.utils import except_wrap, ApiException
from apps.news.models import Post
import util
import datetime

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
        'api/v2/index.html',
        title=u'JSON API',
    )

@mod.route('/posts.json')
@except_wrap
def posts_json():
    year = util.param('year', int)
    month = util.param('month', int)
    q = create_posts_query()
    if year and month:
        current_month = datetime.datetime(year, month, 1)
        next_month = util.add_months(datetime.datetime(year, month, 1), 1)

        q = q.filter(Post.created >= current_month, Post.created < next_month)
        q = util.run_query(q)
    else:
        tag = util.param('tag')
        if tag:
            q = q.filter(Post.tags.IN([tag]))
            q = util.run_query(q)
        else:
            offset = util.param('offset', int) or 0
            limit = util.param('limit', int) or config.ATOM_FEED_LIMIT
            q = util.run_query(q, limit, offset=offset)
    return util.jsonify_model_dbs(q)


def get_post_json(key_id):
    if not key_id:
        raise ApiException('Invalid request: "id" parameter not found.')

    product = None
    if key_id:
        product = Post.retrieve_by_id(key_id)
    if not product and key_id:
        raise ApiException('Post with "%s" == %s not found' % ('id', key_id), status=404)
    return jsonify_model_db(product)

@mod.route('/post.json')
@except_wrap
def post_json():
    return get_post_json(param('id', int))
