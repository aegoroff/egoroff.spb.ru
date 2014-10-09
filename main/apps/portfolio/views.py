# -*- coding: utf-8 -*-
import os
from urlparse import urljoin
import flask
from flask import Blueprint, render_template
from google.appengine.api import memcache
from lxml import etree
import time
from apps.file.models import Folder
import util


class FileResolver(etree.Resolver):
    def resolve(self, url, identifier, context):
        return self.resolve_filename(url, context)


mod = Blueprint(
    'portfolio',
    __name__,
    template_folder='templates',
    url_prefix='/portfolio'
)

APACHE_DOC_PATH_TEMPLATE = 'apache/{0}.xml'


def create_apache_docs():
    config = util.read_json("apache/config.json")
    result = []
    for item in config:
        path = APACHE_DOC_PATH_TEMPLATE.format(item)
        t = time.gmtime(os.path.getmtime(path))
        r = {
            "doc": item,
            "stylesheet": config[item][0],
            "title": config[item][1],
            "descr": config[item][2],
            "keywords": config[item][3],
            "modified": time.strftime("%Y-%m-%d %H:%M:%S", t)
        }
        result.append(r)
    return result


@mod.route('/')
def index():
    folders = Folder.query(Folder.is_public == True).fetch(use_memcache=True, use_cache=True)
    downloads = {}
    for f in folders:
        files = []
        a = lambda key: files.append(key.get())
        map(a, f.files)
        downloads[f.title] = files
    return render_template(
        'portfolio/index.html',
        downloads=downloads,
        apache_docs=create_apache_docs()
    )


@mod.route('/<doc>.html', methods=['GET'])
def get_doc(doc):
    docs = create_apache_docs()
    data = None
    for d in docs:
        if d["doc"] == doc:
            data = d
            break

    if not data:
        return render_template(
            'portfolio/apache_document.html',
            title=doc,
            apache_docs=create_apache_docs()
        )

    content = memcache.get(doc)

    if not content:
        parser = etree.XMLParser(load_dtd=False, dtd_validation=False)
        parser.resolvers.add(FileResolver())

        xml_input = etree.parse(APACHE_DOC_PATH_TEMPLATE.format(doc), parser)
        stylesheet = data["stylesheet"]
        xslt_root = etree.parse('apache/{0}.xsl'.format(stylesheet), parser)
        transform = etree.XSLT(xslt_root)

        content = unicode(transform(xml_input))
        memcache.add(doc, content, 86400)

    return render_template(
        'portfolio/apache_document.html',
        title=data["title"],
        html=content,
        doc=doc,
        meta_description=data["descr"],
        keywords=data["keywords"],
        full_uri=urljoin(flask.request.url_root, flask.url_for('portfolio.get_doc', doc=doc)),
    )
