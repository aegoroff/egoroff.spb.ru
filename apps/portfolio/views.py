# -*- coding: utf-8 -*-

from flask import Blueprint, render_template
from lxml import etree
import main
import site_map


class FileResolver(etree.Resolver):
    def resolve(self, url, identifier, context):
        return self.resolve_filename(url, context)


mod = Blueprint(
    'portfolio',
    __name__,
    template_folder='templates',
    url_prefix='/portfolio'
)

main_section_item = site_map.MAP[0]
apache_section_item = main_section_item[site_map.CHILDS][0]
download_section_item = main_section_item[site_map.CHILDS][1]

@mod.route('/')
def index():
    return render_template(
        'portfolio/index.html',
        parent_id=main_section_item[site_map.ID],
        current_id=main_section_item[site_map.ID],
        breadcrumbs=main.breadcrumbs_home,
        title=main_section_item[site_map.TITLE]
    )


@mod.route('/download/')
def download():
    breadcrumbs =[i for i in main.breadcrumbs_home]
    breadcrumbs.append((main_section_item[site_map.ID], main_section_item[site_map.TITLE]))
    return render_template(
        'portfolio/downloads.html',
        parent_id=main_section_item[site_map.ID],
        current_id=download_section_item[site_map.ID],
        breadcrumbs=breadcrumbs,
        title=download_section_item[site_map.TITLE]
    )


@mod.route('/apache/')
def apache():
    breadcrumbs =[i for i in main.breadcrumbs_home]
    breadcrumbs.append((main_section_item[site_map.ID], main_section_item[site_map.TITLE]))
    return render_template(
        'portfolio/apache.html',
        title=apache_section_item[site_map.TITLE],
        parent_id=main_section_item[site_map.ID],
        current_id=apache_section_item[site_map.ID],
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
    parents = [ main_section_item, apache_section_item ]
    append = lambda item: breadcrumbs.append((item[site_map.ID], item[site_map.TITLE]))
    map(append, parents)

    return render_template(
        'portfolio/apache_document.html',
        title=main.apache_docs[doc][1],
        breadcrumbs=breadcrumbs,
        apache_docs=main.apache_docs,
        html=content
    )
