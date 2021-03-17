#!/usr/bin/env python
# coding: utf-8

import argparse
import json
import os
import platform
import shutil
import sys
from datetime import datetime

from lxml import etree

import config

###############################################################################
# Options
###############################################################################
PARSER = argparse.ArgumentParser()

PARSER.add_argument(
    '-C', '--clean-all', dest='clean_all', action='store_true',
    help='''Cleans all the pip, Node & Bower related tools / libraries and
    updates them to their latest versions''',
)
PARSER.add_argument(
    '-m', '--minify', dest='minify', action='store_true',
    help='compiles files into minified version before deploying'
)

ARGS = PARSER.parse_args()

###############################################################################
# Globals
###############################################################################
BAD_ENDINGS = ['~']
IS_WINDOWS = platform.system() == 'Windows'

###############################################################################
# Directories
###############################################################################
DIR_BOWER_COMPONENTS = 'bower_components'
DIR_MAIN = '.'
DIR_NODE_MODULES = 'node_modules'
DIR_STYLE = 'style'
DIR_SCRIPT = 'script'
DIR_TEMP = 'temp'

DIR_STATIC = os.path.join(DIR_MAIN, 'static')
DIR_APACHE = os.path.join(DIR_MAIN, 'apache')
DIR_TEMPLATES = os.path.join(DIR_MAIN, 'templates')
DIR_TEMPLATES_APACHE = os.path.join(DIR_TEMPLATES, 'apache')

DIR_SRC = os.path.join(DIR_STATIC, 'src')
DIR_SRC_SCRIPT = os.path.join(DIR_SRC, DIR_SCRIPT)
DIR_SRC_STYLE = os.path.join(DIR_SRC, DIR_STYLE)

DIR_DST = os.path.join(DIR_STATIC, 'dst')
DIR_DST_STYLE = os.path.join(DIR_DST, DIR_STYLE)
DIR_DST_SCRIPT = os.path.join(DIR_DST, DIR_SCRIPT)

DIR_EXT = os.path.join(DIR_STATIC, 'ext')
DIR_FONTS = os.path.join(DIR_STATIC, 'fonts')

DIR_MIN = os.path.join(DIR_STATIC, 'min')
DIR_MIN_STYLE = os.path.join(DIR_MIN, DIR_STYLE)
DIR_MIN_SCRIPT = os.path.join(DIR_MIN, DIR_SCRIPT)

FILE_BOWER = 'bower.json'
FILE_PACKAGE = 'package.json'
FILE_NPM_GUARD = os.path.join(DIR_TEMP, 'npm.guard')
FILE_BOWER_GUARD = os.path.join(DIR_TEMP, 'bower.guard')

DIR_BIN = os.path.join(DIR_NODE_MODULES, '.bin')
FILE_COFFEE = os.path.join(DIR_BIN, 'coffee')
FILE_GULP = os.path.join(DIR_BIN, 'gulp')
FILE_LESS = os.path.join(DIR_BIN, 'lessc')
FILE_UGLIFYJS = os.path.join(DIR_BIN, 'uglifyjs')

FILE_UPDATE = os.path.join(DIR_TEMP, 'update.json')


class FileResolver(etree.Resolver):
    def resolve(self, url, identifier, context):
        return self.resolve_filename(url, context)


###############################################################################
# Helpers
###############################################################################
def print_out(script, filename=''):
    timestamp = datetime.now().strftime('%H:%M:%S')
    if not filename:
        filename = '-' * 46
        script = script.rjust(12, '-')
    print('[%s] %12s %s', timestamp, script, filename)


def make_dirs(directory):
    if not os.path.exists(directory):
        os.makedirs(directory)


def remove_file_dir(file_dir):
    if isinstance(file_dir, list) or isinstance(file_dir, tuple):
        for file_ in file_dir:
            remove_file_dir(file_)
    else:
        if not os.path.exists(file_dir):
            return
        if os.path.isdir(file_dir):
            shutil.rmtree(file_dir, ignore_errors=True)
        else:
            os.remove(file_dir)


def clean_files(bad_endings=BAD_ENDINGS, in_dir='.'):
    print_out(
        'CLEAN FILES',
        'Removing files: %s' % ', '.join(['*%s' % e for e in bad_endings]),
    )
    for root, _, files in os.walk(in_dir):
        for filename in files:
            for bad_ending in bad_endings:
                if filename.endswith(bad_ending):
                    remove_file_dir(os.path.join(root, filename))


def merge_files(source, target):
    fout = open(target, 'a', encoding='utf-8')
    for line in open(source, encoding='utf-8'):
        fout.write(line)
    fout.close()


def os_execute(executable, args, source, target, append=False):
    operator = '>>' if append else '>'
    os.system('%s %s %s %s %s' % (executable, args, source, operator, target))


def compile_script(source, target_dir):
    if not os.path.isfile(source):
        print_out('NOT FOUND', source)
        return

    target = source.replace(DIR_SRC_SCRIPT, target_dir).replace('.coffee', '.js')
    if not is_dirty(source, target):
        return
    make_dirs(os.path.dirname(target))
    if not source.endswith('.coffee'):
        print_out('COPYING', source)
        shutil.copy(source, target)
        return
    print_out('COFFEE', source)
    os_execute(FILE_COFFEE, '-cp', source, target)


def compile_style(source, target_dir, check_modified=False):
    if not os.path.isfile(source):
        print_out('NOT FOUND', source)
        return
    if not source.endswith('.less'):
        return

    target = source.replace(DIR_SRC_STYLE, target_dir).replace('.less', '.css')
    if check_modified and not is_style_modified(target):
        return

    minified = ''
    if target_dir == DIR_MIN_STYLE:
        minified = '-x'
        target = target.replace('.css', '.min.css')
        print_out('LESS MIN', source)
    else:
        print_out('LESS', source)

    make_dirs(os.path.dirname(target))
    os_execute(FILE_LESS, minified, source, target)


def is_dirty(source, target):
    if not os.access(target, os.O_RDONLY):
        return True
    return os.stat(source).st_mtime - os.stat(target).st_mtime > 0


def is_style_modified(target):
    for root, _, files in os.walk(DIR_SRC):
        for filename in files:
            path = os.path.join(root, filename)
            if path.endswith('.less') and is_dirty(path, target):
                return True
    return False


def compile_all_dst():
    for source in config.STYLES:
        compile_style(os.path.join(DIR_STATIC, source), DIR_DST_STYLE, True)
    for _, scripts in config.SCRIPTS:
        for source in scripts:
            compile_script(os.path.join(DIR_STATIC, source), DIR_DST_SCRIPT)


def read_json(path):
    with open(path) as f:
        return json.load(f, encoding="UTF-8")


def create_apache_docs():
    conf = read_json(os.path.join(DIR_APACHE, "config.json"))
    result = []
    for item in conf:
        r = {
            "file": item,
            "xml": os.path.join(DIR_APACHE, item + ".xml"),
            "stylesheet": os.path.join(DIR_APACHE, conf[item][0] + ".xsl"),
        }
        result.append(r)
    return result


def compile_xslt():
    docs = create_apache_docs()
    make_dirs(DIR_TEMPLATES_APACHE)
    for d in docs:
        parser = etree.XMLParser(load_dtd=False, dtd_validation=False)
        parser.resolvers.add(FileResolver())

        xml_input = etree.parse(d["xml"], parser)
        stylesheet = d["stylesheet"]
        xslt_root = etree.parse(stylesheet, parser)
        transform = etree.XSLT(xslt_root)
        content = transform(xml_input)
        fout = open(os.path.join(DIR_TEMPLATES_APACHE, d["file"] + ".html"), 'wb')
        fout.write(content)
        fout.close()


def update_path_separators():
    def fixit(path):
        return path.replace('\\', '/').replace('/', os.sep)

    for idx in range(len(config.STYLES)):
        config.STYLES[idx] = fixit(config.STYLES[idx])

    for _, scripts in config.SCRIPTS:
        for idx in range(len(scripts)):
            scripts[idx] = fixit(scripts[idx])


def listdir(directory, split_ext=False):
    try:
        if split_ext:
            return [os.path.splitext(dir_)[0] for dir_ in os.listdir(directory)]
        else:
            return os.listdir(directory)
    except OSError:
        return []


def make_guard(fname, cmd, spec):
    with open(fname, 'w') as guard:
        guard.write('Prevents %s execution if newer than %s' % (cmd, spec))


def guard_is_newer(guard, watched):
    if os.path.exists(guard):
        return os.path.getmtime(guard) > os.path.getmtime(watched)
    return False


def check_if_npm_should_run():
    return not guard_is_newer(FILE_NPM_GUARD, FILE_PACKAGE)


def check_if_bower_should_run():
    return not guard_is_newer(FILE_BOWER_GUARD, FILE_BOWER)


def install_dependencies():
    make_dirs(DIR_TEMP)
    if check_if_npm_should_run():
        make_guard(FILE_NPM_GUARD, 'npm', FILE_PACKAGE)
        os.system('npm install')
    if check_if_bower_should_run():
        make_guard(FILE_BOWER_GUARD, 'bower', FILE_BOWER)
        os.system('"%s" ext' % FILE_GULP)
        os.system('"%s" copy' % FILE_GULP)
        os.system('"%s" fonts' % FILE_GULP)


def update_missing_args():
    if ARGS.clean_all:
        ARGS.clean = True


def uniq(seq):
    seen = set()
    return [e for e in seq if e not in seen and not seen.add(e)]


###############################################################################
# Main
###############################################################################
def run_clean_all():
    print_out('CLEAN ALL')
    remove_file_dir([
        DIR_BOWER_COMPONENTS, DIR_NODE_MODULES, DIR_EXT, DIR_MIN, DIR_DST, DIR_FONTS, DIR_TEMPLATES_APACHE
    ])
    remove_file_dir([
        FILE_NPM_GUARD, FILE_BOWER_GUARD
    ])
    clean_files()


def run_minify():
    print_out('MINIFY')
    clean_files()
    remove_file_dir(DIR_MIN)
    make_dirs(DIR_MIN_SCRIPT)

    compile_xslt()
    for source in config.STYLES:
        compile_style(os.path.join(DIR_STATIC, source), DIR_MIN_STYLE)

    cat, separator = ('type', ',') if IS_WINDOWS else ('cat', ' ')

    for module, scripts in config.SCRIPTS:
        scripts = uniq(scripts)
        coffees = separator.join([
            os.path.join(DIR_STATIC, script)
            for script in scripts if script.endswith('.coffee')
        ])

        pretty_js = os.path.join(DIR_MIN_SCRIPT, '%s.js' % module)
        ugly_js = os.path.join(DIR_MIN_SCRIPT, '%s.min.js' % module)
        print_out('COFFEE MIN', ugly_js)

        if len(coffees):
            os_execute(cat, coffees, ' | %s --compile --stdio' % FILE_COFFEE, pretty_js, append=True)
        for script in scripts:
            if not script.endswith('.js'):
                continue
            script_file = os.path.join(DIR_STATIC, script)
            merge_files(script_file, pretty_js)
        os_execute(FILE_UGLIFYJS, pretty_js, '-cm', ugly_js)
        remove_file_dir(pretty_js)
    print_out('DONE')


def run():
    if len(sys.argv) == 1:
        PARSER.print_help()
        sys.exit(1)

    os.chdir(os.path.dirname(os.path.realpath(__file__)))

    update_path_separators()
    update_missing_args()

    if ARGS.clean_all:
        run_clean_all()

    if ARGS.minify:
        install_dependencies()
        run_minify()


if __name__ == '__main__':
    run()
