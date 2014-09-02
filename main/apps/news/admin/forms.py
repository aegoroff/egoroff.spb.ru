# -*- coding: utf-8 -*-
import datetime

from flask.ext import wtf
import wtforms


class TagListField(wtforms.Field):
    widget = wtforms.StringField()

    def _value(self):
        if self.data:
            return u', '.join(self.data)
        else:
            return u''

    def process_formdata(self, valuelist):
        if valuelist:
            self.data = [x.strip() for x in valuelist[0].split(',')]
        else:
            self.data = []


class PostForm(wtf.Form):
    created = wtforms.DateTimeField(
        u'Создано',
        description=u'Дата создания',
        default=datetime.datetime.now(),
        validators=[wtforms.validators.required()]
    )
    title = wtforms.StringField(
        u'Заголовок',
        description=u'Введите заголовок записи',
        validators=[wtforms.validators.required()]
    )
    is_public = wtforms.BooleanField(
        u'Публичная?',
        description=u'Отметьте, чтобы показывать новость на сайте',
        default=False,
        validators=[wtforms.validators.optional()]
    )
    tags = TagListField(
        u'Тэги',
        description=u'Список тегов',
        validators=[wtforms.validators.optional()]
    )
    short_text = wtforms.TextAreaField(
        u'Краткое описание',
        description=u'Введите краткое описание',
        validators=[wtforms.validators.required()]
    )
    text = wtforms.TextAreaField(
        u'Текст',
        description=u'Введите основной текст записи',
        validators=[wtforms.validators.optional()]
    )