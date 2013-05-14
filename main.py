#!/usr/bin/env python
#
# Copyright 2007 Google Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
import logging
from lxml import etree
import os
from google.appengine.ext import ndb
from google.appengine.api import users
import webapp2
import jinja2

jinja_environment = jinja2.Environment(
    loader=jinja2.FileSystemLoader(os.path.dirname(__file__)),
    extensions=['jinja2.ext.autoescape'])


class MenuItem(ndb.Model):
  title = ndb.TextProperty()
  link = ndb.TextProperty()
  active = ndb.BooleanProperty()
  description = ndb.TextProperty()


main_menu = [
    MenuItem(title="Portfolio", link="/portfolio/"),
    MenuItem(title="Opinions", link="/opinions/"),
    MenuItem(title="News", link="/news/")
]


class MainHandler(webapp2.RequestHandler):
    def get(self):
        user = users.get_current_user()
        template = jinja_environment.get_template('templates/main.html')

        self.response.out.write(template.render(
            main_menu=main_menu,
            page_title="Main page",
            site_name="egoroff.spb.ru",
            user=user
            ))


class PortfolioHandler(webapp2.RequestHandler):
    def get(self):
        user = users.get_current_user()
        template = jinja_environment.get_template('templates/portfolio.html')

        xml_input = etree.XML(open('mod_rewrite.xml', 'r').read())
        xslt_root = etree.XML(open('apache_module.xsl', 'r').read())
        transform = etree.XSLT(xslt_root)

        content = unicode(transform(xml_input))

        self.response.out.write(template.render(
            main_menu=main_menu,
            page_title="Portfolio",
            site_name="egoroff.spb.ru",
            user=user,
            html=content
            ))


class OpinionsHandler(webapp2.RequestHandler):
    def get(self):
        user = users.get_current_user()
        template = jinja_environment.get_template('templates/opinions.html')

        self.response.out.write(template.render(
            main_menu=main_menu,
            page_title="Opinions",
            site_name="egoroff.spb.ru",
            user=user
            ))


class NewsHandler(webapp2.RequestHandler):
    def get(self):
        user = users.get_current_user()
        template = jinja_environment.get_template('templates/news.html')

        self.response.out.write(template.render(
            main_menu=main_menu,
            page_title="News",
            site_name="egoroff.spb.ru",
            user=user
            ))

app = webapp2.WSGIApplication([
    ('/', MainHandler),
    ('/portfolio/', PortfolioHandler),
    ('/opinions/', OpinionsHandler),
    ('/news/', NewsHandler),
], debug=True)
