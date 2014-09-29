# coding: utf-8

import os
import site_map


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
  RECAPTCHA_PUBLIC_KEY = CONFIG_DB.recaptcha_public_key
  RECAPTCHA_PRIVATE_KEY = CONFIG_DB.recaptcha_private_key

DEFAULT_DB_LIMIT = 64
ATOM_FEED_LIMIT = 20
BRAND_NAME = 'Admin Config'

SITE = site_map.MAP

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
        'ext/js/moment/moment-with-locales.js',
        'ext/js/nprogress/nprogress.js',
        'ext/js/knockout/knockout.js',
        'ext/js/knockoutmapping/knockout.mapping.js',
        'ext/js/bootstrap/alert.js',
        'ext/js/bootstrap/button.js',
        'ext/js/bootstrap/transition.js',
        'ext/js/bootstrap/collapse.js',
        'ext/js/bootstrap/dropdown.js',
        'ext/js/bootstrap/tooltip.js',
        'ext/js/bootstrapvalidator/bootstrapValidator.js',
        'ext/js/bootstrapvalidator/language/ru_RU.js',
        'ext/js/moment-timezone/moment-timezone-with-data.js',
        'ext/js/jstz/jstz.js',
        'ext/js/redactor/redactor.js',
        'src/script/syntax-highlighter/shCore.js',
        'src/script/syntax-highlighter/shBrushCpp.js',
        'src/script/syntax-highlighter/shBrushCSharp.js',
        'src/script/syntax-highlighter/shBrushXml.js',
        'src/script/syntax-highlighter/shBrushJScript.js',
        'src/script/syntax-highlighter/shBrushParser.js',
        'src/script/syntax-highlighter/shBrushSql.js',
        'src/script/syntax-highlighter/shBrushNasm8086.js',
        'src/script/syntax-highlighter/shBrushIL.js',
        'src/script/syntax-highlighter/shBrushHashQuery.js',
        'src/script/syntax-highlighter/shBrushPython.js',
        'src/script/syntax-highlighter/shBrushCss.js',
        'src/script/syntax-highlighter/shBrushLess.js',
        'src/script/jquery.plugins/jquery.form.js',
        'src/script/jquery.plugins/jquery.metadata.js',
        'src/script/jquery.plugins/jquery.mosaic.js',
        'src/script/jquery.plugins/jquery.tablesorter.js',
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
        'src/script/site/blog.coffee',
        'src/script/site/search.coffee',
      ]),
  ]
