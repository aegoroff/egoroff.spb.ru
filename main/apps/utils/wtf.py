# -*- coding: utf-8 -*-

from flaskext import wtf

class CommaField(wtf.TextAreaField):
    def _value(self):
        if self.data:
            return u', '.join(self.data)
        else:
            return u''

    def process_formdata(self, valuelist):
        if valuelist and not (len(valuelist) == 1 and valuelist[0] ==u''):
            self.data = [x.strip() for x in valuelist[0].split(',')]
        else:
            self.data = []