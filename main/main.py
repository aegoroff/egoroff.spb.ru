# coding: utf-8
import json

import logging

import sys
from urlparse import urljoin
import site_map
import typographus

from flask.ext import wtf
from flask import request, current_app
import flask
import wtforms

import config
import util

app = flask.Flask(__name__)
app.config.from_object(config)
app.jinja_env.line_statement_prefix = '#'
app.jinja_env.line_comment_prefix = '##'
app.jinja_env.globals.update(
    check_form_fields=util.check_form_fields,
    is_iterable=util.is_iterable,
    slugify=util.slugify,
    update_query_argument=util.update_query_argument,
  )

app.jinja_env.add_extension('apps.compressinja.html.SelectiveHtmlCompressor')

import admin
import auth
import model
import task
import user

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


@app.context_processor
def inject_context_data():
    return site_map.inject_context_data()


@app.template_filter('typo')
def typo(s):
    return typographus.typo(s)


app.jinja_env.filters["typo"] = typo


if config.DEVELOPMENT:
  from werkzeug import debug
  app.wsgi_app = debug.DebuggedApplication(app.wsgi_app, evalex=True)


###############################################################################
# Main page
###############################################################################
@app.route('/')
def welcome():
    limit = 5
    posts = util.run_query(create_posts_query(), limit)
    return flask.render_template(
        'welcome.html',
        html_class='welcome',
        apache_docs=create_apache_docs(),
        posts=posts
    )

class Search():
  key = config.CONFIG_DB.search_api_key
  cx = '006596644808879549558:dwgc4vapbog'

@app.route('/search/')
def search():
    action_uri='https://www.googleapis.com/customsearch/v1'
    if config.DEVELOPMENT:
        action_uri = flask.url_for('do_search')
    return flask.render_template(
      'search.html',
      html_class='search',
      search=Search(),
      action_uri=action_uri,
      has_json=True
    )

@app.route('/dosearch/')
def do_search():
    obj = util.readJson("customsearchresponse.json")
    return current_app.response_class(json.dumps(obj), mimetype='application/json')



###############################################################################
# Sitemap stuff
###############################################################################
@app.route('/sitemap.xml')
def sitemap():
    create = lambda path: urljoin(flask.request.url_root, path)
    full_uri = lambda marker: create(flask.url_for(marker))

    pages = [util.create_page(create(''), "weekly")]

    for item in site_map.MAP[0][site_map.CHILDS]:
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


###############################################################################
# Profile stuff
###############################################################################
class ProfileUpdateForm(wtf.Form):
  name = wtforms.StringField(u'Имя',
      [wtforms.validators.required()], filters=[util.strip_filter],
    )
  email = wtforms.StringField(u'Электропочта',
      [wtforms.validators.optional(), wtforms.validators.email()],
      filters=[util.email_filter],
    )


@app.route('/_s/profile/', endpoint='profile_service')
@app.route('/profile/', methods=['GET', 'POST'])
@auth.login_required
def profile():
  user_db = auth.current_user_db()
  form = ProfileUpdateForm(obj=user_db)

  if form.validate_on_submit():
    email = form.email.data
    if email and not user_db.is_email_available(email, user_db.key):
      form.email.errors.append(u'Этот email уже занят')

    if not form.errors:
      send_verification = not user_db.token or user_db.email != email
      form.populate_obj(user_db)
      if send_verification:
        user_db.verified = False
        task.verify_email_notification(user_db)
      user_db.put()
      return flask.redirect(flask.url_for('welcome'))

  if flask.request.path.startswith('/_s/'):
    return util.jsonify_model_db(user_db)

  return flask.render_template(
      'profile.html',
      title=user_db.name,
      html_class='profile',
      form=form,
      user_db=user_db,
      has_json=True,
    )


###############################################################################
# Feedback
###############################################################################
class FeedbackForm(wtf.Form):
  subject = wtforms.StringField(u'Тема',
      [wtforms.validators.required()], filters=[util.strip_filter],
    )
  message = wtforms.TextAreaField(u'Сообщение',
      [wtforms.validators.required()], filters=[util.strip_filter],
    )
  email = wtforms.StringField(u'Ваша электропочта (необязательно)',
      [wtforms.validators.optional(), wtforms.validators.email()],
      filters=[util.email_filter],
    )


@app.route('/feedback/', methods=['GET', 'POST'])
def feedback():
  if not config.CONFIG_DB.feedback_email:
    return flask.abort(418)

  form = FeedbackForm(obj=auth.current_user_db())
  if form.validate_on_submit():
    body = '%s\n\n%s' % (form.message.data, form.email.data)
    kwargs = {'reply_to': form.email.data} if form.email.data else {}
    task.send_mail_notification(form.subject.data, body, **kwargs)
    flask.flash(u'Спасибо за обратную связь!', category='success')
    return flask.redirect(flask.url_for('welcome'))

  return flask.render_template(
      'feedback.html',
      html_class='feedback',
      form=form,
    )


###############################################################################
# Warmup request
###############################################################################
@app.route('/_ah/warmup')
def warmup():
  # TODO: put your warmup code here
  return 'success'


###############################################################################
# Error Handling
###############################################################################
@app.errorhandler(400)  # Bad Request
@app.errorhandler(401)  # Unauthorized
@app.errorhandler(403)  # Forbidden
@app.errorhandler(404)  # Not Found
@app.errorhandler(405)  # Method Not Allowed
@app.errorhandler(410)  # Gone
@app.errorhandler(418)  # I'm a Teapot
@app.errorhandler(500)  # Internal Server Error
def error_handler(e):
  logging.exception(e)
  try:
    e.code
  except AttributeError:
    e.code = 500
    e.name = 'Internal Server Error'

  if flask.request.path.startswith('/_s/'):
    return util.jsonpify({
        'status': 'error',
        'error_code': e.code,
        'error_name': util.slugify(e.name),
        'error_message': e.name,
        'error_class': e.__class__.__name__,
      }), e.code

  return flask.render_template(
      'error.html',
      title= u'Ошибка %d (%s)!!!' % (e.code, e.name),
      html_class='error-page',
      error=e,
    ), e.code


if config.PRODUCTION:
  @app.errorhandler(Exception)
  def production_error_handler(e):
    return error_handler(e)
