import os
import site_map

try:
  # This part is surrounded in try/except because the this config.py file is
  # also used in the build.py script which is used to compile/minify the client
  # side files (*.less, *.coffee, *.js) and is not aware of the GAE
  import model
  from datetime import datetime
  CONFIG_DB = model.Config.get_master_db()
  SECRET_KEY = CONFIG_DB.flask_secret_key.encode('ascii')
  CURRENT_VERSION_ID = os.environ.get('CURRENT_VERSION_ID', None)
  CURRENT_VERSION_NAME = CURRENT_VERSION_ID.split('.')[0]
  CURRENT_VERSION_TIMESTAMP = long(CURRENT_VERSION_ID.split('.')[1]) >> 28
  CURRENT_VERSION_DATE = datetime.fromtimestamp(CURRENT_VERSION_TIMESTAMP)
except:
  pass

PRODUCTION = os.environ.get('SERVER_SOFTWARE', '').startswith('Google App Engine')
DEVELOPMENT = not PRODUCTION
DEBUG = DEVELOPMENT

DEFAULT_DB_LIMIT = 64
ATOM_FEED_LIMIT = 20

SITE = site_map.MAP

################################################################################
# Client modules, also used by the build.py script.
################################################################################
STYLES = [
    'src/less/style.less',
    'src/less/apache.less',
    'src/less/shCoreEclipse.less',
    'src/less/tags.less',
  ]

SCRIPTS_MODULES = [
    'libs',
    'jquery.plugins',
    'moment',
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
      'src/coffee/site/news.async.reader.coffee',
    ],
    'home': [
      'src/coffee/site/rotate.post.coffee',
    ],
  }
