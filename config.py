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

if os.environ.get('SERVER_SOFTWARE', '').startswith('Google App Engine'):
  DEVELOPMENT = False
else:
  DEVELOPMENT = True

PRODUCTION = not DEVELOPMENT
DEBUG = DEVELOPMENT

DEFAULT_DB_LIMIT = 64

SITE = site_map.MAP

################################################################################
# Client modules, also used by the build.py script.
################################################################################
STYLES = [
    'src/less/style.less',
    'src/less/apache.less',
  ]

SCRIPTS_MODULES = [
    'libs',
    'scripts',
    'jquery.plugins',
    'admin'
  ]

SCRIPTS = {
    'libs': [
      'lib/jquery.js',
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
    'scripts': [
      'src/coffee/common/util.coffee',
      'src/coffee/common/service.coffee',

      'src/coffee/site/app.coffee',
      'src/coffee/site/profile.coffee',
      'src/coffee/site/admin.coffee',
    ],
  }
