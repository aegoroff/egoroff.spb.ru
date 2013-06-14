# -*- coding: utf-8 -*-
import os

from flask import Blueprint, render_template
from lxml import etree
import time
from apps.file.models import Folder, File
import site_map
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
    config = util.readJson("apache/config.json")
    result = []
    for item in config:
        path = APACHE_DOC_PATH_TEMPLATE.format(item)
        t = time.gmtime(os.path.getmtime(path))
        r = {
            "doc": item,
            "stylesheet": config[item][0],
            "title": config[item][1],
            "descr": config[item][2],
            "modified": time.strftime("%Y-%m-%d %H:%M:%S", t)
        }
        result.append(r)
    return result


main_section_item = site_map.MAP[0]

@mod.route('/')
def index():
    folders = Folder.query(Folder.is_public == True)
    downloads = {}
    for f in folders:
        files = []
        a = lambda key: files.append(key.get())
        map(a, f.files)
        downloads[f.title] = files
    return render_template(
        'portfolio/index.html',
        parent_id=main_section_item[site_map.ID],
        title=main_section_item[site_map.TITLE],
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

    parser = etree.XMLParser(load_dtd=False, dtd_validation=False)
    parser.resolvers.add(FileResolver())

    xml_input = etree.parse(APACHE_DOC_PATH_TEMPLATE.format(doc), parser)
    stylesheet = data["stylesheet"]
    xslt_root = etree.parse('apache/{0}.xsl'.format(stylesheet), parser)
    transform = etree.XSLT(xslt_root)

    content = unicode(transform(xml_input))

    return render_template(
        'portfolio/apache_document.html',
        title=data["title"],
        html=content
    )
