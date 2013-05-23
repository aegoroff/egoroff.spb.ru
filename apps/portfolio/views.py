# -*- coding: utf-8 -*-

from flask import Blueprint, render_template
import json
from lxml import etree


def readJson(path):
    with open(path) as f:
        return json.load(f, encoding="UTF-8")


apache_docs = readJson("apache/config.json")


class FileResolver(etree.Resolver):
    def resolve(self, url, identifier, context):
        return self.resolve_filename(url, context)


mod = Blueprint(
    'portfolio',
    __name__,
    template_folder='templates',
    url_prefix='/portfolio'
)


@mod.route('/')
def index():
    return render_template(
        'portfolio/index.html',
        title=u"Портфель",
        apache_docs=apache_docs
    )


@mod.route('/<doc>.html', methods=['GET'])
def get_doc(doc):
    if doc not in apache_docs:
        return

    parser = etree.XMLParser(load_dtd=False, dtd_validation=False)
    parser.resolvers.add(FileResolver())

    xml_input = etree.parse('apache/{0}.xml'.format(doc), parser)
    stylesheet = apache_docs[doc][0]
    xslt_root = etree.parse('apache/{0}.xsl'.format(stylesheet), parser)
    transform = etree.XSLT(xslt_root)

    content = unicode(transform(xml_input))

    return render_template(
        'portfolio/apache.html',
        title=apache_docs[doc][1],
        apache_docs=apache_docs,
        html=content
    )
