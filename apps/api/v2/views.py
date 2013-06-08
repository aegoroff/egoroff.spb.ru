# -*- coding: utf-8 -*-

from flask import Blueprint, render_template, current_app, request
import json
from util import param, jsonify_model_db
from apps.api.utils import except_wrap, ApiException, jsonify_success
from datetime import datetime
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
    is_public = param('is_public', bool)
    if is_public is not None and not is_public:
        posts_query = Post.query(Post.is_public != True)
    else:
        posts_query = Post.query(Post.is_public == True)

    posts_dbs, more_cursor = util.retrieve_dbs(
      posts_query,
      limit=util.param('limit', int),
      cursor=util.param('cursor'),
      order=util.param('order') or '-created',
      name=util.param('name'),
      admin=util.param('admin', bool),
    )

    return util.jsonify_model_dbs(posts_dbs, more_cursor)

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