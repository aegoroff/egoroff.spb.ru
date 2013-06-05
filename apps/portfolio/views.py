# -*- coding: utf-8 -*-
import os

from flask import Blueprint, render_template
from lxml import etree
import time
from apps.file.models import Folder, File
import main
import site_map
import util


class FileResolver(etree.Resolver):
    def resolve(self, url, identifier, context):
        return self.resolve_filename(url, context)


mod = Blueprint(
    'portfolio',
    __name__,
    template_folder='templates'
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


@mod.route('/portfolio/<int:key_id>.html', methods=['GET'])
def portfolio_files(key_id):
    remapping = {
        1: 19002,
        3: 24001,
        5: 23001,
        6: 21002,
        7: 22001,
        8: 21001,
        9: 13004,
        10: 6007,
        11: 18001,
        12: 12004,
        13: 12003,
        14: 1006,
        15: 6005,
        16: 16002,
        17: 12002,
        18: 6006,
        19: 17001,
        21: 11004,
        22: 9004,
        23: 13003,
        24: 16001,
        25: 9003,
        26: 14005,
        27: 5004,
        28: 11003,
        29: 15001,
        30: 8002
    }
    return util.redirect(key_id, remapping)


main_section_item = site_map.MAP[0]


@mod.route('/portfolio/download/')
@mod.route('/portfolio/flickr/')
@mod.route('/apache/')
@mod.route('/portfolio/')
def index():
    folders = Folder.query()
    downloads = {}
    for f in folders:
        if not f.is_public:
            continue
        files = []
        a = lambda key: files.append(key.get())
        map(a, f.files)
        downloads[f.title] = files
    return render_template(
        'portfolio/index.html',
        parent_id=main_section_item[site_map.ID],
        current_id=main_section_item[site_map.ID],
        breadcrumbs=main.create_breadcrumbs([]),
        title=main_section_item[site_map.TITLE],
        downloads=downloads,
        apache_docs=create_apache_docs()
    )


@mod.route('/portfolio/<doc>.html', methods=['GET'])
@mod.route('/apache/<doc>.html', methods=['GET'])
@mod.route('/portfolio/apache/<doc>.html', methods=['GET'])
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
            breadcrumbs=main.create_breadcrumbs([main_section_item]),
            html=u"<p>Пока ничего нет</p>"
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
        breadcrumbs=main.create_breadcrumbs([main_section_item]),
        html=content
    )
