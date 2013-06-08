# -*- coding: utf-8 -*-

from apps.file.models import File
from google.appengine.ext import ndb
from model import Base
from werkzeug.wrappers import cached_property


class Post(Base):
    title = ndb.StringProperty(verbose_name=u'Заголовок', required=True)
    is_public = ndb.BooleanProperty(verbose_name=u'Публичная?')
    short_text = ndb.TextProperty(verbose_name=u'Краткое описание', required=True)
    text = ndb.TextProperty(verbose_name=u'Основной текст')
    tags = ndb.StringProperty(verbose_name=u'Теги', repeated=True)
    _PROPERTIES = set([
      'key', 'id', 'created', 'modified', 'created_ago', 'modified_ago', 'title', 'short_text'
    ])

class Tag(object):
  """Holds tag properties """
  def __init__(self, title, tag_level):
    self._title = title
    self._tag_level = tag_level

  @property
  def title(self):
    return self._title

  @property
  def tag_level(self):
    return self._tag_level