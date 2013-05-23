# -*- coding: utf-8 -*-

from flaskext import wtf

class FolderForm(wtf.Form):
    title = wtf.TextField(
        u'Название папки',
        validators=[wtf.validators.required()]
    )
    is_public = wtf.BooleanField(
        u'Публичная?',
        default=False,
        validators=[wtf.validators.optional()]
    )

class FileForm(wtf.Form):
    file = wtf.FileField(
        u'Файл',
        description=u'Выберите файл для загрузки',
        validators=[wtf.validators.required()]
    )
    description = wtf.TextAreaField(
        u'Описание',
        description=u'Введите краткое описание для файла',
        validators=[wtf.validators.optional()]
    )