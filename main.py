# -*- coding: utf-8 -*-
import random

import sys
from urlparse import urljoin
import site_map
import typographus


sys.path.insert(0, 'lib.zip')

from google.appengine.api import mail
import flask
from flaskext import wtf
from flask import request, current_app
import config

app = flask.Flask(__name__)
app.config.from_object(config)
app.jinja_env.line_statement_prefix = '#'
app.jinja_env.add_extension('jinja2htmlcompress.SelectiveHTMLCompress')

import auth
import util
import model
import admin

from apps.file.views import mod as file_view

app.register_blueprint(file_view)

from apps.file.admin.views import mod as file_admin_mod

app.register_blueprint(file_admin_mod)

from apps.portfolio.views import mod as portfolio_mod, create_apache_docs

app.register_blueprint(portfolio_mod)

from apps.news.views import mod as news_mod, create_posts_query

app.register_blueprint(news_mod)

from apps.news.admin.views import mod as news_admin_mod

app.register_blueprint(news_admin_mod)

from apps.compatibility.views import mod as compatibility_mod

app.register_blueprint(compatibility_mod)

from apps.api.v2.views import mod as api_mod

app.register_blueprint(api_mod)


def create_breadcrumbs(parents):
    breadcrumbs = [('welcome', u'Главная', 'icon-home')]
    append = lambda item: breadcrumbs.append((item[site_map.ID], item[site_map.TITLE], item[site_map.CLASS]))
    map(append, parents)
    return breadcrumbs


def current_section():
    for section in site_map.MAP:
        index = section[site_map.ID]
        section_uri = flask.url_for(index)
        if section_uri in request.path:
            return section, section_uri
    return None, None


@app.context_processor
def inject_current_section():
    curr, uri = current_section()
    if curr:
        return dict(current_id=curr[site_map.ID])
    return dict(current_id="")


@app.context_processor
def inject_breadcrumbs():
    if request.path == '/':
        return dict(breadcrumbs=None)
    curr, uri = current_section()
    if curr:
        if request.path == uri and (not request.query_string or request.query_string == ''):
            return dict(breadcrumbs=create_breadcrumbs([]))
        return dict(breadcrumbs=create_breadcrumbs([curr]))
    return dict(breadcrumbs=create_breadcrumbs([]))


@app.template_filter('time_ago')
def time_ago(timestamp):
    return util.format_datetime_ago(timestamp)


app.jinja_env.filters["time_ago"] = time_ago


@app.template_filter('typo')
def typo(s):
    return typographus.typo(s)


app.jinja_env.filters["typo"] = typo


@app.route('/')
def welcome():
    limit = 5
    posts = util.run_query(create_posts_query(), limit)
    return flask.render_template(
        'welcome.html',
        html_class='welcome',
        apache_docs=create_apache_docs(),
        posts=posts,
        rnd=random.randint(0,limit-1)
    )

@app.route('/search/')
def search():
    title=u'Поиск'
    q = util.param('q')
    if q:
        title = u"Результаты по запросу: {0}".format(q)
    return flask.render_template(
        'search.html',
        title=title,
        html_class='search'
    )

@app.route('/sitemap.xml')
def sitemap():
    create = lambda path: urljoin(flask.request.url_root, path)
    full_uri = lambda marker: create(flask.url_for(marker))

    pages = [util.create_page(create(''), "weekly")]

    for item in site_map.MAP:
        if item[site_map.ID] == 'search': # filter search because it disabled it robots.txt
            continue
        p = util.create_page(full_uri(item[site_map.ID]), "weekly", '0.7')
        pages.append(p)

    apache_docs = create_apache_docs()

    for d in apache_docs:
        loc = create(urljoin(flask.url_for('portfolio.index'), '{0}.html'.format(d['doc'])))
        p = util.create_page(loc, "yearly")
        pages.append(p)

    keys = util.run_query(create_posts_query(), None, keys_only=True)
    for k in keys:
        loc = create(urljoin(flask.url_for('news.index'), '{0}.html'.format(k.id())))
        p = util.create_page(loc, "yearly")
        pages.append(p)

    sitemap_xml = flask.render_template('sitemap.html', pages=pages , mimetype='text/xml')
    response = flask.make_response(sitemap_xml)
    response.headers["Content-Type"] = "application/xml"
    return response

################################################################################
# Profile stuff
################################################################################
class ProfileUpdateForm(wtf.Form):
    name = wtf.TextField(u'Имя', [wtf.validators.required()])
    email = wtf.TextField(u'Электропочта', [
        wtf.validators.optional(),
        wtf.validators.email(u'Это не похоже на электропочту :)'),
    ])


@app.route('/_s/profile/', endpoint='profile_service')
@app.route('/profile/', methods=['GET', 'POST'], endpoint='profile')
@auth.login_required
def profile():
    form = ProfileUpdateForm()
    user_db = auth.current_user_db()
    if form.validate_on_submit():
        user_db.name = form.name.data
        user_db.email = form.email.data.lower()
        user_db.put()
        return flask.redirect(flask.url_for('welcome'))
    if not form.errors:
        form.name.data = user_db.name
        form.email.data = user_db.email or ''

    if flask.request.path.startswith('/_s/'):
        return util.jsonify_model_db(user_db)

    return flask.render_template(
        'profile.html',
        title=u'Профиль',
        html_class='profile',
        form=form,
        user_db=user_db,
    )


################################################################################
# Feedback
################################################################################
class FeedbackForm(wtf.Form):
    subject = wtf.TextField(u'Тема', [wtf.validators.required()])
    message = wtf.TextAreaField(u'Сообщение', [wtf.validators.required()])
    email = wtf.TextField(u'Электропочта (необязательно)', [
        wtf.validators.optional(),
        wtf.validators.email(u'Это не похоже на электропочту :)'),
    ])


@app.route('/feedback/', methods=['GET', 'POST'])
@auth.login_required
def feedback():
    form = FeedbackForm()
    if form.validate_on_submit():
        mail.send_mail(
            sender=config.CONFIG_DB.feedback_email,
            to=config.CONFIG_DB.feedback_email,
            subject='[%s] %s' % (
                config.CONFIG_DB.brand_name,
                form.subject.data,
            ),
            reply_to=form.email.data or config.CONFIG_DB.feedback_email,
            body='%s\n\n%s' % (form.message.data, form.email.data)
        )
        flask.flash(u'Ушло! Спасибо за мнение!', category='success')
        return flask.redirect(flask.url_for('welcome'))
    if not form.errors and auth.current_user_id() > 0:
        form.email.data = auth.current_user_db().email

    return flask.render_template(
        'feedback.html',
        title=u'Фидбек',
        html_class='feedback',
        form=form,
    )


################################################################################
# User Stuff
################################################################################
@app.route('/_s/user/', endpoint='user_list_service')
@app.route('/user/', endpoint='user_list')
@auth.admin_required
def user_list():
    user_dbs, more_cursor = util.retrieve_dbs(
        model.User.query(),
        limit=util.param('limit', int),
        cursor=util.param('cursor'),
        order=util.param('order') or '-created',
        name=util.param('name'),
        admin=util.param('admin', bool),
    )

    if flask.request.path.startswith('/_s/'):
        return util.jsonify_model_dbs(user_dbs, more_cursor)

    return flask.render_template(
        'user_list.html',
        html_class='user',
        title=u'Пользователи',
        user_dbs=user_dbs,
        more_url=util.generate_more_url(more_cursor),
    )


################################################################################
# Error Handling
################################################################################
@app.errorhandler(400)
@app.errorhandler(401)
@app.errorhandler(403)
@app.errorhandler(404)
@app.errorhandler(410)
@app.errorhandler(418)
@app.errorhandler(500)
def error_handler(e):
    try:
        e.code
    except:
        class e(object):
            code = 500
            name = 'Internal Server Error'

    if flask.request.path.startswith('/_s/'):
        return flask.jsonify({
            'status': 'error',
            'error_code': e.code,
            'error_name': e.name.lower().replace(' ', '_'),
            'error_message': e.name,
        }), e.code

    return flask.render_template(
        'error.html',
        title='Error %d (%s)!!!' % (e.code, e.name),
        html_class='error-page',
        error=e,
    ), e.code
