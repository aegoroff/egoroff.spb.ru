# -*- coding: utf-8 -*-
import datetime

from flaskext import wtf


class GMT3(datetime.tzinfo):
    def utcoffset(self, dt):
        return datetime.timedelta(hours=3) + self.dst(dt)

    def dst(self, dt):
        d = datetime.datetime(dt.year, 4, 1)
        self.dston = d - datetime.timedelta(days=d.weekday() + 1)
        self.datetime = datetime.datetime(dt.year, 11, 1)
        d = self.datetime
        self.dstoff = d - datetime.timedelta(days=d.weekday() + 1)
        if self.dston <= dt.replace(tzinfo=None) < self.dstoff:
            return datetime.timedelta(hours=1)
        else:
            return datetime.timedelta(0)

    def tzname(self, dt):
        return "GMT +3"


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
        default=datetime.datetime.now(GMT3()),
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