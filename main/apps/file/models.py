# -*- coding: utf-8 -*-

from model import Base
from google.appengine.ext import ndb
from google.appengine.ext.blobstore import BlobInfo
from os import path
import uuid
from google.appengine.api import images
from flask import url_for
from auth import current_user_key
import units

IMAGE_TYPES = ('image/bmp', 'image/jpeg', 'image/png',
    'image/gif', 'image/tiff', 'image/x-icon')

class File(Base):
    uid = ndb.StringProperty()
    title = ndb.StringProperty(verbose_name=u'Название файла', indexed=False)
    description = ndb.TextProperty(verbose_name=u'Описание файла', indexed=False)
    owner = ndb.KeyProperty()
    is_public = ndb.BooleanProperty(default=False, verbose_name=u'Публичный?')
    url = ndb.StringProperty(verbose_name=u'URL')

    blob_key = ndb.BlobKeyProperty(verbose_name=u'Ключ файла')
    filename = ndb.StringProperty(verbose_name=u'Оригинальное название файла', indexed=False)
    size = ndb.IntegerProperty(verbose_name=u'Размер файла', indexed=False)
    human_readable_size = ndb.ComputedProperty(lambda self: units.format4human(self.size))
    content_type = ndb.StringProperty(verbose_name=u'Тип файла')

    ext = ndb.ComputedProperty(lambda self: path.splitext(self.filename)[1][1:] if self.filename is not None else '')
    is_image = ndb.ComputedProperty(lambda self: self.content_type in IMAGE_TYPES)
    title_filename = ndb.ComputedProperty(
        lambda self: u'%s.%s'%(self.title, self.ext) if self.title and self.ext else self.filename if self.filename else self.uid)

    def get_cached_url(self, force=False):
        if not self.blob_key:
            return ''
        if not self.url or force:
            blob_info = BlobInfo.get(self.blob_key)
            if blob_info:
                if blob_info.content_type in IMAGE_TYPES:
                    cached_url = images.get_serving_url(self.blob_key)
                else:
                    cached_url = url_for('file.get', file_key=self.uid, _external=True)
                self.url = cached_url
        return self.url

    @classmethod
    def create(cls, blob_key=None, **kwargs):
        return cls(
            uid=str(uuid.uuid1()).replace('-', ''),
            blob_key=blob_key,
            **kwargs)

    def _pre_put_hook(self):
        if not self.uid:
            self.uid = str(uuid.uuid1()).replace('-', '')
        self.owner = current_user_key()
        self.url = self.get_cached_url(force=True)

    def delete_blob(self):
        if self.blob_key:
            blob_info = BlobInfo.get(self.blob_key)
            if blob_info:
                blob_info.delete()

    @classmethod
    def _pre_delete_hook(cls, key):
        file_ = key.get()
        if file_:
            file_.delete_blob()

    @classmethod
    def is_image_type(cls, content_type):
        return content_type in IMAGE_TYPES



class Folder(Base):
    title = ndb.StringProperty(verbose_name=u'Название', indexed=False)
    is_public = ndb.BooleanProperty(verbose_name=u'Публичная?', default=False)
    files = ndb.KeyProperty(File, verbose_name=u'Файлы', repeated=True)

    @classmethod
    def _pre_delete_hook(cls, key):
        folder = key.get()
        if folder and folder.files:
            for f in folder.files:
                f.delete()