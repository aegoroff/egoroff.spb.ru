# coding: utf-8

import os


PRODUCTION = os.environ.get('SERVER_SOFTWARE', '').startswith('Google App Eng')
DEBUG = DEVELOPMENT = not PRODUCTION

try:
  # This part is surrounded in try/except because the config.py file is
  # also used in the run.py script which is used to compile/minify the client
  # side files (*.less, *.coffee, *.js) and is not aware of the GAE
  from google.appengine.api import app_identity
  APPLICATION_ID = app_identity.get_application_id()
except (ImportError, AttributeError):
  pass
else:
  from datetime import datetime
  CURRENT_VERSION_ID = os.environ.get('CURRENT_VERSION_ID')
  CURRENT_VERSION_NAME = CURRENT_VERSION_ID.split('.')[0]
  CURRENT_VERSION_TIMESTAMP = long(CURRENT_VERSION_ID.split('.')[1]) >> 28
  if DEVELOPMENT:
    import calendar
    CURRENT_VERSION_TIMESTAMP = calendar.timegm(datetime.utcnow().timetuple())
  CURRENT_VERSION_DATE = datetime.utcfromtimestamp(CURRENT_VERSION_TIMESTAMP)

  import model

  CONFIG_DB = model.Config.get_master_db()
  SECRET_KEY = CONFIG_DB.flask_secret_key.encode('ascii')

DEFAULT_DB_LIMIT = 64
ATOM_FEED_LIMIT = 20
BRAND_NAME = 'Admin Config'

###############################################################################
# Client modules, also used by the run.py script.
###############################################################################
STYLES = [
    'src/style/style.less',
  ]

SCRIPTS = [
    ('libs', [
        'ext/js/jquery/jquery.js',
        'ext/js/jquery-ui/jquery-ui.js',
        'ext/js/momentjs/moment.js',
        'ext/js/nprogress/nprogress.js',
        'ext/js/knockout/knockout.js',
        'ext/js/knockoutmapping/knockout.mapping.js',
        'ext/js/bootstrap/alert.js',
        'ext/js/bootstrap/button.js',
        'ext/js/bootstrap/transition.js',
        'ext/js/bootstrap/collapse.js',
        'ext/js/bootstrap/dropdown.js',
        'ext/js/bootstrap/tooltip.js',
        'ext/js/moment-timezone/moment-timezone-with-data.js',
        'ext/js/jstz/jstz.js',
      ]),
    ('scripts', [
        'src/script/common/service.coffee',
        'src/script/common/util.coffee',
        'src/script/site/app.coffee',
        'src/script/site/admin.coffee',
        'src/script/site/profile.coffee',
        'src/script/site/signin.coffee',
        'src/script/site/user.coffee',
        'src/script/site/format.date.coffee',
        'src/script/site/rotate.post.coffee',
        'src/script/site/blog.model.coffee',
        'src/script/site/blog.async.reader.coffee',
      ]),
  ]
  
'''
 STYLES = [
    'src/less/style.less',
    'src/less/apache.less',
    'src/less/shCoreEclipse.less',
    'src/less/tags.less',
    'src/less/nprogress.less',
  ]

SCRIPTS_MODULES = [
    'libs',
    'jquery.plugins',
    'moment',
    'knockout',
    'scripts',
    'syntax-highlighter',
    'admin'
  ]

SCRIPTS = {
    'libs': [
      'lib/jquery.js',
      'lib/jquery-ui-1.10.3.custom.js',
      'lib/bootstrap/js/bootstrap-alert.js',
      'lib/bootstrap/js/bootstrap-button.js',
      'lib/bootstrap/js/bootstrap-collapse.js',
      'lib/bootstrap/js/bootstrap-dropdown.js',
      'lib/bootstrap/js/bootstrap-tooltip.js',
      'lib/nprogress.js',
    ],
    'jquery.plugins': [
        'lib/jquery.mosaic.js',
        'lib/jquery.form.js',
        'lib/jquery.tablesorter.js',
        'lib/jquery.metadata.js',
    ],
    'admin':[
        'lib/redactor.js'
    ],
    'moment':[
        'lib/moment-with-locales.js',
        'lib/moment-timezone-with-data.js',
        'lib/jstz-1.0.4.min.js',
    ],
    'knockout':[
        'lib/knockout-3.2.0.debug.js',
        'lib/knockout.mapping-latest.debug.js',
    ],
    'syntax-highlighter': [
        'lib/syntax-highlighter/shCore.js',
        'lib/syntax-highlighter/shBrushCpp.js',
        'lib/syntax-highlighter/shBrushCSharp.js',
        'lib/syntax-highlighter/shBrushXml.js',
        'lib/syntax-highlighter/shBrushJScript.js',
        'lib/syntax-highlighter/shBrushParser.js',
        'lib/syntax-highlighter/shBrushSql.js',
        'lib/syntax-highlighter/shBrushNasm8086.js',
        'lib/syntax-highlighter/shBrushIL.js',
        'lib/syntax-highlighter/shBrushHashQuery.js',
    ],
    'scripts': [
      'src/coffee/common/util.coffee',
      'src/coffee/common/service.coffee',
      'src/coffee/site/app.coffee',
      'src/coffee/site/profile.coffee',
      'src/coffee/site/admin.coffee',
      'src/coffee/site/format.date.coffee',
      'src/coffee/site/blog.model.coffee',
      'src/coffee/site/blog.async.reader.coffee',
    ],
    'home': [
      'src/coffee/site/rotate.post.coffee',
    ],
  }
'''
