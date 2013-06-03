# -*- coding: utf-8 -*-

from flask import Blueprint, render_template
from lxml import etree
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
    template_folder='templates',
    url_prefix='/portfolio'
)

@mod.route('/<int:key_id>.html', methods=['GET'])
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

@mod.route('/download/')
@mod.route('/apache/')
@mod.route('/')
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
        breadcrumbs=main.breadcrumbs_home,
        title=main_section_item[site_map.TITLE],
        downloads=downloads,
        apache_docs=main.apache_docs
    )


def create_breadcrumbs():
    breadcrumbs = [i for i in main.breadcrumbs_home]
    parents = [main_section_item]
    append = lambda item: breadcrumbs.append((item[site_map.ID], item[site_map.TITLE]))
    map(append, parents)
    return breadcrumbs


@mod.route('/<doc>.html', methods=['GET'])
@mod.route('/apache/<doc>.html', methods=['GET'])
def get_doc(doc):
    if doc not in main.apache_docs:
        return render_template(
            'portfolio/apache_document.html',
            title=doc,
            breadcrumbs=create_breadcrumbs(),
            apache_docs=main.apache_docs,
            html=u"<p>Пока ничего нет</p>"
        )

    parser = etree.XMLParser(load_dtd=False, dtd_validation=False)
    parser.resolvers.add(FileResolver())

    xml_input = etree.parse('apache/{0}.xml'.format(doc), parser)
    stylesheet = main.apache_docs[doc][0]
    xslt_root = etree.parse('apache/{0}.xsl'.format(stylesheet), parser)
    transform = etree.XSLT(xslt_root)

    content = unicode(transform(xml_input))

    return render_template(
        'portfolio/apache_document.html',
        title=main.apache_docs[doc][1],
        breadcrumbs=create_breadcrumbs(),
        apache_docs=main.apache_docs,
        html=content
    )
