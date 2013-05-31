# -*- coding: utf-8 -*-
import datetime

from flaskext import wtf


class PostForm(wtf.Form):
    created = wtf.DateTimeField(
        u'Создано',
        description=u'Дата создания',
        default=datetime.datetime.now(),
        validators=[wtf.validators.required()]
    )
    title = wtf.TextField(
        u'Заголовок',
        description=u'Введите заголовок записи',
        validators=[wtf.validators.required()]
    )
    is_public = wtf.BooleanField(
        u'Публичная?',
        description=u'Отметьте, чтобы показывать новость на сайте',
        default=False,
        validators=[wtf.validators.optional()]
    )
    short_text = wtf.TextAreaField(
        u'Краткое описание',
        description=u'Введите краткое описание',
        validators=[wtf.validators.required()]
    )
    text = wtf.TextAreaField(
        u'Текст',
        description=u'Введите основной текст записи',
        validators=[wtf.validators.optional()]
    )