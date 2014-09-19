# coding: utf-8

from flask.ext import wtf
from google.appengine.api import app_identity
import flask
import wtforms

import auth
import config
import model
import util

from main import app


class ConfigUpdateForm(wtf.Form):
  analytics_id = wtforms.StringField('Tracking ID', filters=[util.strip_filter])
  announcement_html = wtforms.TextAreaField('Announcement HTML', filters=[util.strip_filter])
  announcement_type = wtforms.SelectField('Announcement Type', choices=[(t, t.title()) for t in model.Config.announcement_type._choices])
  brand_name = wtforms.StringField('Brand Name', [wtforms.validators.required()], filters=[util.strip_filter])
  check_unique_email = wtforms.BooleanField('Check for the uniqueness of the verified emails')
  facebook_app_id = wtforms.StringField('App ID', filters=[util.strip_filter])
  facebook_app_secret = wtforms.StringField('App Secret', filters=[util.strip_filter])
  feedback_email = wtforms.StringField('Feedback Email', [wtforms.validators.optional(), wtforms.validators.email()], filters=[util.email_filter])
  flask_secret_key = wtforms.StringField('Secret Key', [wtforms.validators.optional()], filters=[util.strip_filter])
  notify_on_new_user = wtforms.BooleanField('Send an email notification when a user signs up')
  twitter_consumer_key = wtforms.StringField('Consumer Key', filters=[util.strip_filter])
  twitter_consumer_secret = wtforms.StringField('Consumer Secret', filters=[util.strip_filter])
  verify_email = wtforms.BooleanField('Verify user emails')
  search_api_key = wtforms.StringField('Search API key', filters=[util.strip_filter])


@app.route('/_s/admin/config/', endpoint='admin_config_update_service')
@app.route('/admin/config/', methods=['GET', 'POST'])
@auth.admin_required
def admin_config_update():
  config_db = model.Config.get_master_db()
  form = ConfigUpdateForm(obj=config_db)
  if form.validate_on_submit():
    form.populate_obj(config_db)
    if not config_db.flask_secret_key:
      config_db.flask_secret_key = util.uuid()
    config_db.put()
    reload(config)
    app.config.update(CONFIG_DB=config_db)
    return flask.redirect(flask.url_for('welcome'))

  if flask.request.path.startswith('/_s/'):
    return util.jsonify_model_db(config_db)

  instances_url = None
  if config.PRODUCTION:
    instances_url = '%s?app_id=%s&version_id=%s' % (
        'https://appengine.google.com/instances',
        app_identity.get_application_id(),
        config.CURRENT_VERSION_ID,
      )

  return flask.render_template(
      'admin/config_update.html',
      title='Admin Config',
      html_class='admin-config',
      form=form,
      config_db=config_db,
      instances_url=instances_url,
      has_json=True,
    )


class Search():
  q = ''
  key = config.CONFIG_DB.search_api_key
  cx = '006596644808879549558:dwgc4vapbog'

class SearchForm(wtf.Form):
    q = wtforms.StringField('Query', filters=[util.strip_filter])
    key = wtforms.HiddenField()
    cx = wtforms.HiddenField()

@app.route('/admin/search/', methods=['GET', 'POST'])
@auth.admin_required
def admin_custom_search():
    s = Search()
    form = SearchForm(obj=s)
    action_uri = flask.url_for('do_search')
    return flask.render_template(
      'admin/custom_search.html',
      title='Custom search',
      html_class='admin-config',
      form=form,
      action_uri=action_uri,
      has_json=True,
    )