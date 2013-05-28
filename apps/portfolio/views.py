# -*- coding: utf-8 -*-

from flask import Blueprint, render_template
from lxml import etree
import main


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
        breadcrumbs=main.breadcrumbs_home,
        title=u"Портфель"
    )

@mod.route('/apache/')
def apache():
    breadcrumbs =[i for i in main.breadcrumbs_home]
    breadcrumbs.append(('portfolio.index', u"Портфель"))
    return render_template(
        'portfolio/apache.html',
        title=u"Про апачей",
        breadcrumbs=breadcrumbs,
        apache_docs=main.apache_docs
    )


@mod.route('/apache/<doc>.html', methods=['GET'])
def get_doc(doc):
    if doc not in main.apache_docs:
        return

    parser = etree.XMLParser(load_dtd=False, dtd_validation=False)
    parser.resolvers.add(FileResolver())

    xml_input = etree.parse('apache/{0}.xml'.format(doc), parser)
    stylesheet = main.apache_docs[doc][0]
    xslt_root = etree.parse('apache/{0}.xsl'.format(stylesheet), parser)
    transform = etree.XSLT(xslt_root)

    content = unicode(transform(xml_input))

    breadcrumbs =[i for i in main.breadcrumbs_home]
    breadcrumbs.append(('portfolio.index', u"Портфель"))
    breadcrumbs.append(('portfolio.apache', u"Про апачей"))
    return render_template(
        'portfolio/apache_document.html',
        title=main.apache_docs[doc][1],
        breadcrumbs=breadcrumbs,
        apache_docs=main.apache_docs,
        html=content
    )
