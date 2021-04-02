# coding: utf-8

###############################################################################
# Client modules, also used by the build.py script.
###############################################################################
STYLES = [
    'src/style/style.less',
    'src/style/adminstyle.less',
  ]

SCRIPTS = [
    ('libs', [
        'ext/jquery/dist/jquery.js',
        'ext/jquery-ui/jquery-ui.js',
        'ext/moment/min/moment-with-locales.js',
        'ext/nprogress/nprogress.js',
        'ext/knockout/dist/knockout.js',
        'ext/knockoutmapping/knockout.mapping.js',
        'ext/bootstrap/js/alert.js',
        'ext/bootstrap/js/button.js',
        'ext/bootstrap/js/transition.js',
        'ext/bootstrap/js/collapse.js',
        'ext/bootstrap/js/dropdown.js',
        'ext/bootstrap/js/tooltip.js',
        'ext/bootstrapvalidator/dist/js/bootstrapValidator.js',
        'ext/bootstrapvalidator/dist/js/language/ru_RU.js',
        'ext/moment-timezone/builds/moment-timezone-with-data.js',
        'ext/jstz/jstz.js',
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
      ]),
    ('admin', [
        'ext/redactor/redactor/redactor.js',
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
