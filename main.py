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
import os
from google.appengine.ext import ndb
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


class MainHandler(webapp2.RequestHandler):
    def get(self):
        template = jinja_environment.get_template('templates/main.html')

        home = MenuItem(title="Portfolio", link="/portfolio/", active=True)
        about = MenuItem(title="Opinions", link="/opinions/")
        contact = MenuItem(title="News", link="/news/")

        main_menu = [home, about, contact]

        self.response.out.write(template.render(
            main_menu=main_menu,
            page_title="Main page",
            site_name="egoroff.spb.ru"
            ))

app = webapp2.WSGIApplication([
    ('/', MainHandler)
], debug=True)
