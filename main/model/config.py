# coding: utf-8

from __future__ import absolute_import

from google.appengine.ext import ndb

import config
import model
import util


class Config(model.Base):
  analytics_id = ndb.StringProperty(default='')
  announcement_html = ndb.TextProperty(default='')
  announcement_type = ndb.StringProperty(default='info', choices=[
      'info', 'warning', 'success', 'danger',
    ])
  anonymous_recaptcha = ndb.BooleanProperty(default=False)
  brand_name = ndb.StringProperty(default=config.APPLICATION_ID)
  check_unique_email = ndb.BooleanProperty(default=True)
  facebook_app_id = ndb.StringProperty(default='')
  facebook_app_secret = ndb.StringProperty(default='')
  feedback_email = ndb.StringProperty(default='')
  flask_secret_key = ndb.StringProperty(default=util.uuid())
  notify_on_new_user = ndb.BooleanProperty(default=True)
  recaptcha_private_key = ndb.StringProperty(default='')
  recaptcha_public_key = ndb.StringProperty(default='')
  twitter_consumer_key = ndb.StringProperty(default='')
  twitter_consumer_secret = ndb.StringProperty(default='')
  verify_email = ndb.BooleanProperty(default=True)
  search_api_key = ndb.StringProperty(default='')
  google_site_id = ndb.StringProperty(default='')

  @property
  def has_anonymous_recaptcha(self):
    return bool(self.anonymous_recaptcha and self.has_recaptcha)

  @property
  def has_facebook(self):
    return bool(self.facebook_app_id and self.facebook_app_secret)

  @property
  def has_recaptcha(self):
    return bool(self.recaptcha_private_key and self.recaptcha_public_key)

  @property
  def has_twitter(self):
    return bool(self.twitter_consumer_key and self.twitter_consumer_secret)

  @property
  def has_search_api(self):
    return bool(self.search_api_key)

  _PROPERTIES = model.Base._PROPERTIES.union({
      'analytics_id',
      'announcement_html',
      'announcement_type',
      'anonymous_recaptcha',
      'brand_name',
      'check_unique_email',
      'facebook_app_id',
      'facebook_app_secret',
      'feedback_email',
      'flask_secret_key',
      'notify_on_new_user',
      'twitter_consumer_key',
      'twitter_consumer_secret',
      'verify_email',
      'search_api_key',
      'google_site_id'
    })

  @classmethod
  def get_master_db(cls):
    return cls.get_or_insert('master')
