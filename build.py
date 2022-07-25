#!/usr/bin/env python3
# coding: utf-8

import argparse
import json
import os
import platform
import shutil
from datetime import datetime

from lxml import etree

###############################################################################
# Options
###############################################################################
PARSER = argparse.ArgumentParser()

PARSER.add_argument(
    '-C', '--clean-all', dest='clean_all', action='store_true',
    help='''Cleans all the Node & Bower related tools / libraries and
    updates them to their latest versions''',
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
DIR_MAIN = '.'

DIR_APACHE = os.path.join(DIR_MAIN, 'apache')
DIR_TEMPLATES = os.path.join(DIR_MAIN, 'templates')
DIR_TEMPLATES_APACHE = os.path.join(DIR_TEMPLATES, 'apache')


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


def read_json(path):
    with open(path) as f:
        return json.load(f)


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


###############################################################################
# Main
###############################################################################
def run_clean_all():
    print_out('CLEAN ALL')
    remove_file_dir([
        DIR_TEMPLATES_APACHE
    ])


def run():
    os.chdir(os.path.dirname(os.path.realpath(__file__)))

    if ARGS.clean_all:
        run_clean_all()

    compile_xslt()


if __name__ == '__main__':
    run()
