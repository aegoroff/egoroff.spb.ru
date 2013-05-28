# -*- coding: utf-8 -*-

from flask import Blueprint, render_template
import main
import site_map


mod = Blueprint(
    'opinions',
    __name__,
    template_folder='templates',
    url_prefix='/opinions'
)

main_section_item = site_map.MAP[1]

@mod.route('/')
def index():
    return render_template(
        'opinions/index.html',
        parent_id=main_section_item[site_map.ID],
        current_id=main_section_item[site_map.ID],
        breadcrumbs=main.breadcrumbs_home,
        title=main_section_item[site_map.TITLE]
    )