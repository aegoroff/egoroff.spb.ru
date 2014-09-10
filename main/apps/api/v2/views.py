# -*- coding: utf-8 -*-
import os
import random
from apps.news.views import get_posts_ids, create_posts_query
from apps.utils import blobstore_handlers
import config

from flask import Blueprint, render_template, current_app, request
import json
from util import param, jsonify_model_db
from apps.api.utils import except_wrap, ApiException
import datetime
from apps.news.models import Post
import util
import flask
from auth import admin_required
from apps.file.models import Folder, File
from apps.utils.blobstore import get_uploads
from datetime import datetime

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
    if not product:
        if key_id:
            raise ApiException('Post with "%s" == %s not found' % ('id', key_id), status=404)
    return jsonify_model_db(product)

@mod.route('/post.json')
@except_wrap
def post_json():
    return get_post_json(param('id', int))


@mod.route('/add_file/', methods=['POST'])
@admin_required
def add_blob_content():

    response_object = {
      'status': 'success',
      'now': datetime.utcnow().isoformat(),
      'filelink': None,
    }

    folder = Folder.retrieve_by_id(3001)
    if not folder:
        response_object['status'] = 'failure'
        return util.jsonpify(response_object)
    upload_files = get_uploads(flask.request, 'file')
    if len(upload_files):
        blob_info = upload_files[0]
        if blob_info.size:
            f = File.create(
                blob_info.key(),
                size=blob_info.size,
                filename=os.path.basename(blob_info.filename.replace('\\','/')),
                content_type=blob_info.content_type)
            f.put()
            if f.get_cached_url():
                folder.files.append(f.key)
                folder.put()
                response_object['filelink'] = f.get_cached_url()
        else:
            blob_info.delete()
    return util.jsonpify(response_object)