# -*- coding: utf-8 -*-

from flask.ext import wtf
import wtforms

class FolderForm(wtf.Form):
    title = wtforms.StringField(
        u'Название папки',
        validators=[wtforms.validators.required()]
    )
    is_public = wtforms.BooleanField(
        u'Публичная?',
        default=False,
        validators=[wtforms.validators.optional()]
    )

class FileForm(wtf.Form):
    file = wtforms.FileField(
        u'Файл',
        description=u'Выберите файл для загрузки',
        validators=[wtforms.validators.required()]
    )
    description = wtforms.TextAreaField(
        u'Описание',
        description=u'Введите краткое описание для файла',
        validators=[wtforms.validators.optional()]
    )