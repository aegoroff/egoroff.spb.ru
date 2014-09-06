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
    file = wtforms.FileField(u'Файл', [wtforms.validators.required()], tuple(), u'Выберите файл для загрузки')
    description = wtforms.TextAreaField(
        u'Описание',
        validators=[wtforms.validators.optional()],
        description=u'Введите краткое описание для файла'
    )