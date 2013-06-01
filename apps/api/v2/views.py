# -*- coding: utf-8 -*-

from flask import Blueprint, render_template, current_app, request
from apps.product.models import Product, Category
import json
from util import param, jsonify_model_db
from apps.api.utils import except_wrap, ApiException, jsonify_success
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
    available_product = Product.query(Product.is_available==True, Product.id_1c!='').get()
    available_category = Category.query().get()
    return render_template(
        'api/v2/index.html',
        available_product=available_product,
        available_category=available_category
    )

@mod.route('/to_sync.json')
@except_wrap
def get_to_sync():
    id_1c = param('id_1c')
    if id_1c is not None:
        products = Product.query(Product.to_sync == True).fetch(projection=[Product.id_1c])
        return jsonify_success([key.id_1c for key in products])

    products = Product.query(Product.to_sync == True).fetch(keys_only=True)
    return jsonify_success([key.id() for key in products])


@mod.route('/products_count.json')
@except_wrap
def products_count_json():
    is_available = param('available', bool)
    if is_available is not None and is_available:
        return jsonify_success(Product.query(Product.is_available==True).count())
    if is_available is not None and not is_available:
        return jsonify_success(Product.query(Product.is_available!=True).count())
    return jsonify_success(Product.query().count())

@mod.route('/products.json')
@except_wrap
def products_json():
    is_available = param('available', bool)
    if is_available is not None:
        if is_available:
            products_q = Product.query(Product.is_available == True)
        else:
            products_q = Product.query(Product.is_available != True)
    else:
        products_q = Product.query()

    is_1c = param('id_1c')
    if is_1c is not None:
        return jsonify_success(
            [key.id_1c for key in products_q.fetch(projection=[Product.id_1c])]
        )
    return jsonify_success(
        [key.id() for key in products_q.fetch(keys_only=True)]
    )

@mod.route('/products_dates.json')
@except_wrap
def products_dates_json():
    date1 = param("from_date")
    date2 = param('to_date')
    if date1 is None:
        ApiException('Invalid request: parameter "from_date" not found.')
    date1 = datetime.strptime(date1, '%Y-%m-%d')
    if date2 is None:
        date2 = datetime.now()
    else:
        date2 = datetime.strptime(date2, '%Y-%m-%d')
    ApiException(date2)
    is_new = param('only_new')
    if is_new is None:
        products_q = Product.query(
            Product.modified >= date1,
            Product.modified <= date2).fetch(keys_only=True)
    else:
        products_q = Product.query(
            Product.created >= date1,
            Product.created <= date2).fetch(keys_only=True)
    return jsonify_success([key.id() for key in products_q])


@mod.route('/product.json')
@except_wrap
def product_json():
    key_id = param('id')
    id_1c = param('id_1c')
    if not key_id and not id_1c:
        raise ApiException('Invalid request: "id" or "id_1c" params not found.')
    if key_id is not None and id_1c is not None:
        raise ApiException('Invalid request: "id" and "id_1c" together.')

    product = None
    if key_id:
        product = Product.retrieve_by_id(key_id)
    if id_1c:
        product = Product.query(Product.id_1c==id_1c).get()
    if not product:
        if key_id:
            raise ApiException('Product with "%s" == %s not found' % ('id', key_id), status=404)
        raise ApiException('Product with "%s" == %s not found' % ('id_1c', id_1c), status=404)
    return jsonify_model_db(product)

@mod.route('/categories_count.json')
@except_wrap
def categories_count_json():
    return jsonify_success(Category.query().count())

@mod.route('/categories.json')
@except_wrap
def categories_json():
    categories = [key.id() for key in Category.query().fetch(keys_only=True)]
    return jsonify_success(categories)

@mod.route('/category.json')
@except_wrap
def category_json():
    key_id = param('id')
    if not key_id:
        raise ApiException('Invalid request: category "id" not found.')
    category = Category.retrieve_by_id(key_id)
    if not category:
        raise ApiException(
            'Invalid request: Category with "id" == %s not found'
            % key_id
        )
    id_1c = param('id_1c')
    if id_1c is not None:
        category._PROPERTIES = category._PROPERTIES.union(['products_by_id_1c'])
        return jsonify_model_db(category)
    return jsonify_model_db(category)
