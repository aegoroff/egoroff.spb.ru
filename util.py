# -*- coding: utf-8 -*-
import calendar
import json

from google.appengine.datastore.datastore_query import Cursor
from google.appengine.ext import ndb
from google.appengine.ext import blobstore
import flask

from uuid import uuid4
from datetime import datetime
import urllib
import re
import unicodedata

import config


################################################################################
# Request Parameters
################################################################################
def param(name, cast=None):
  '''Returs query parameter by its name, and optionaly casts it to given type.
  Always returns None if the parameter is missing
  '''
  res = None
  if flask.request.json:
    return flask.request.json.get(name, None)

  if res is None:
    res = flask.request.args.get(name, None)
  if res is None and flask.request.form:
    res = flask.request.form.get(name, None)

  if cast and res:
    if cast == bool:
      return res.lower() in ['true', 'yes', '1']
    return cast(res)
  return res


def get_next_url():
  next = param('next')
  if next:
    return next
  referrer = flask.request.referrer
  if referrer and referrer.startswith(flask.request.host_url):
    return referrer
  return flask.url_for('welcome')


################################################################################
# Model manipulations
################################################################################
def retrieve_dbs(query, order=None, limit=None, cursor=None, **filters):
  ''' Retrieves entities from datastore, by applying cursor pagination
  and equality filters. Returns dbs and more cursor value
  '''
  limit = limit or config.DEFAULT_DB_LIMIT
  cursor = Cursor.from_websafe_string(cursor) if cursor else None
  model_class = ndb.Model._kind_map[query.kind]
  if order:
    for o in order.split(','):
      if o.startswith('-'):
        query = query.order(-model_class._properties[o[1:]])
      else:
        query = query.order(model_class._properties[o])

  for prop in filters:
    if filters.get(prop, None) is None:
      continue
    if type(filters[prop]) == list:
      for value in filters[prop]:
        query = query.filter(model_class._properties[prop] == value)
    else:
      query = query.filter(model_class._properties[prop] == filters[prop])

  model_dbs, more_cursor, more = query.fetch_page(limit, start_cursor=cursor)
  more_cursor = more_cursor.to_websafe_string() if more else None
  return list(model_dbs), more_cursor


################################################################################
# JSON Response Helpers
################################################################################
def jsonify_model_dbs(model_dbs, more_cursor=None):
  '''Return a response of a list of dbs as JSON service result
  '''
  result_objects = []
  for model_db in model_dbs:
    result_objects.append(model_db_to_object(model_db))

  response_object = {
      'status': 'success',
      'count': len(result_objects),
      'now': format_datetime_utc(datetime.utcnow()),
      'result': result_objects,
    }
  if more_cursor:
    response_object['more_cursor'] = more_cursor
    response_object['more_url'] = generate_more_url(more_cursor)
  response = flask.jsonify(response_object)
  return response


def jsonify_model_db(model_db):
  '''Return respons of a db as JSON service result
  '''
  result_object = model_db_to_object(model_db)
  response = flask.jsonify({
      'status': 'success',
      'now': format_datetime_utc(datetime.utcnow()),
      'result': result_object,
    })
  return response


def model_db_to_object(model_db):
  model_db_object = {}
  for prop in model_db._PROPERTIES:
    if prop == 'id':
      try:
        value = json_value(getattr(model_db, 'key', None).id())
      except:
        value = None
    else:
      value = json_value(getattr(model_db, prop, None))
    if value is not None:
      model_db_object[prop] = value
  return model_db_object


def json_value(value):
  if type(value) == datetime:
    return format_datetime_utc(value)
  if type(value) == ndb.Key:
    return value.urlsafe()
  if type(value) == blobstore.BlobKey:
    return urllib.quote(str(value))
  if type(value) == ndb.GeoPt:
    return '%s,%s' % (value.lat, value.lon)
  if type(value) == list:
    return [json_value(v) for v in value]
  if type(value) == long:
    # Big numbers are sent as strings for accuracy in JavaScript
    if value > 9007199254740992 or value < -9007199254740992:
      return str(value)
  if isinstance(value, ndb.Model):
    return model_db_to_object(value)
  return value


################################################################################
# Helpers
################################################################################
def generate_more_url(more_cursor, base_url=None, cursor_name='cursor'):
  '''Substitutes or alters the current request url with a new cursor parameter
  for next page of results
  '''
  if not more_cursor:
    return None
  base_url = base_url or flask.request.base_url
  args = flask.request.args.to_dict()
  args[cursor_name] = more_cursor
  return '%s?%s' % (base_url, urllib.urlencode(args))


def uuid():
  ''' Generates universal unique identifier
  '''
  return str(uuid4()).replace('-', '')


_slugify_strip_re = re.compile(r'[^\w\s-]')
_slugify_hyphenate_re = re.compile(r'[-\s]+')


def slugify(value):
  if not isinstance(value, unicode):
    value = unicode(value)
  value = unicodedata.normalize('NFKD', value).encode('ascii', 'ignore')
  value = unicode(_slugify_strip_re.sub('', value).strip().lower())
  return _slugify_hyphenate_re.sub('-', value)


################################################################################
# In Time
################################################################################
def format_datetime_utc(datetime):
  return datetime.strftime('%Y-%m-%d %H:%M:%S UTC')


SECOND = 1
MINUTE = 60 * SECOND
HOUR = 60 * MINUTE
DAY = 24 * HOUR
MONTH = 30 * DAY
YEAR = 365 * DAY


def declension(number, nominative, genitiveSingular, genitivePlural):
    if number < 0:
        number = 0 - number
    lastDigit = number % 10
    lastTwoDigits = number % 100
    if lastDigit == 1 and lastTwoDigits != 11:
        return nominative
    if lastDigit == 2 and lastTwoDigits != 12 or lastDigit == 3 and lastTwoDigits != 13 or lastDigit == 4 and lastTwoDigits != 14:
        return genitiveSingular
    return genitivePlural


def format_datetime_ago(timestamp):
  delta = datetime.utcnow() - timestamp
  seconds = delta.seconds + delta.days * DAY
  minutes = 1.0 * seconds / MINUTE
  hours = 1.0 * seconds / HOUR
  days = 1.0 * seconds / DAY
  months = 1.0 * seconds / MONTH
  years = 1.0 * seconds / YEAR

  ago_template = u'%0.0f %s назад'
  if seconds < 0:
    return u'только что'
  if seconds < 1 * MINUTE:
    return u'%d %s назад' % (seconds, declension(seconds, u"секунду", u"секунды", u"секунд"))
  if seconds < 2 * MINUTE:
    return u'минуту назад'
  if seconds < 45 * MINUTE:
    return ago_template % (minutes, declension(int(minutes), u"минуту", u"минуты", u"минут"))
  if seconds < 90 * MINUTE:
    return u'час назад'
  if seconds < 24 * HOUR:
    return ago_template % (hours, declension(int(hours), u"час", u"часа", u"часов"))
  if seconds < 48 * HOUR:
    return u'вчера'
  if seconds < 30 * DAY:
    return ago_template % (days, declension(int(days), u"день", u"дня", u"дней"))
  if seconds < 12 * MONTH:
    return ago_template % (months, declension(int(months), u"месяц", u"месяца", u"месяцев"))
  else:
    return ago_template % (years, declension(int(years), u"год", u"года", u"лет"))


def readJson(path):
    with open(path) as f:
        return json.load(f, encoding="UTF-8")


def add_months(source_date, months):
    month = source_date.month - 1 + months
    year = source_date.year + month / 12
    month = month % 12 + 1
    day = min(source_date.day,calendar.monthrange(year,month)[1])
    return datetime(year,month,day)

import xml.etree.ElementTree as ET

def create_site_map_xml(pages):
    urlset = ET.Element('urlset')
    urlset.set("xmlns", 'http://www.sitemaps.org/schemas/sitemap/0.9')
    for page in pages:
        url = ET.SubElement(urlset, 'url')
        loc = ET.SubElement(url, 'loc')
        loc.text = page["loc"]
        changefreq = ET.SubElement(url, 'changefreq')
        changefreq.text = page["changefreq"]
        priority = ET.SubElement(url, 'priority')
        priority.text = page["priority"]

    return ET.tostring(urlset, encoding="UTF-8")