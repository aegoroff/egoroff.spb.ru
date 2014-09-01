# -*- coding: utf-8 -*-
import datetime

from flaskext import wtf


class TagListField(wtf.Field):
    widget = wtf.TextInput()

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
    tags = TagListField(
        u'Тэги',
        description=u'Список тегов',
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