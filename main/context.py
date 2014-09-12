# coding: utf-8

import flask
import site_map
from flask import request
from main import app

__author__ = 'egr'


def create_breadcrumbs(breadcrumbs, parents):
    append = lambda item: breadcrumbs.append((item[site_map.ID], item[site_map.TITLE], item[site_map.CLASS]))
    map(append, parents)
    return breadcrumbs


def current_section():
    for root in site_map.MAP:
        section_uri = flask.url_for(root[site_map.ID])
        if section_uri == request.path:
            return root, None, section_uri

        for child in root[site_map.CHILDS]:
            child_uri = flask.url_for(child[site_map.ID])
            if child_uri in request.path:
                return root, child, child_uri
    return site_map.MAP[0], None, request.path


@app.context_processor
def inject_context_data():
    root, curr, uri = current_section()

    current_id = ''
    root_id = ''
    breadcrumbs = None
    sections = None
    if curr:
        current_id = curr[site_map.ID]
    if root:
        root_id = root[site_map.ID]

    for s in site_map.MAP:
        if s[site_map.ID] == root_id:
            sections = s[site_map.CHILDS]
            break

    if request.path != flask.url_for('welcome'):
        start = [(root[site_map.ID], root[site_map.TITLE], root[site_map.CLASS])]
        if curr:
            if request.path == uri and (not request.query_string or request.query_string == ''):
                breadcrumbs = create_breadcrumbs(start, [])
            else:
                breadcrumbs = create_breadcrumbs(start, [curr])
        else:
            breadcrumbs = create_breadcrumbs(start, [])
    return dict(
        current_id=current_id,
        root_id=root_id,
        breadcrumbs=breadcrumbs,
        sections=sections,
        main_section_item=curr)