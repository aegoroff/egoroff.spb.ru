# -*- coding: utf-8 -*-
from flaskext import wtf


class WriteKeyForm(wtf.Form):
    email = wtf.TextField(u'Email', validators=[wtf.validators.email()])
