# -*- coding: utf-8 -*-

import flask
from flask import request

__author__ = 'egr'

ID = "id"
CLASS = "class"
TITLE = "title"
CHILDS = "childs"


def create_breadcrumbs(breadcrumbs, parents):
    append = lambda item: breadcrumbs.append((item[ID], item[TITLE], item[CLASS]))
    map(append, parents)
    return breadcrumbs


def current_section():
    for root in MAP:
        section_uri = flask.url_for(root[ID])
        if section_uri == request.path:
            return root, None, section_uri

        for child in root[CHILDS]:
            child_uri = flask.url_for(child[ID])
            if child_uri in request.path:
                return root, child, child_uri
    return MAP[0], None, request.path


def inject_context_data():
    root, curr, uri = current_section()

    current_id = ''
    root_id = ''
    breadcrumbs = None
    sections = None
    if curr:
        current_id = curr[ID]
    if root:
        root_id = root[ID]

    for s in MAP:
        if s[ID] == root_id:
            sections = s[CHILDS]
            break

    if request.path != flask.url_for('welcome'):
        start = [(root[ID], root[TITLE], root[CLASS])]
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
        current_section=curr)

MAP = [
    {
        ID: "welcome",
        CLASS: "fa fa-home",
        TITLE: u'Главная',
        CHILDS : [
            {
                ID: "portfolio.index",
                CLASS: "fa fa-briefcase",
                TITLE: u"Портфель"
            },
            {
                ID: "news.index",
                CLASS: "fa fa-book",
                TITLE: u"Блог"
            },
            {
                ID: "search",
                CLASS: "fa fa-search",
                TITLE: u"Поиск"
            },
            {
                ID: "news.recent_feed",
                CLASS: "fa fa-rss",
                TITLE: u"RSS"
            },
        ]
    },
    {
        ID: "admin_config_update",
        CLASS: "fa fa-fw fa-cog",
        TITLE: u'Админка',
        CHILDS : [
            {
                ID: "admin.news.index",
                CLASS: "fa fa-book",
                TITLE: u"Блог"
            },
            {
                ID: "admin.file.index",
                CLASS: "fa fa-cloud-upload",
                TITLE: u"Файло"
            },
        ]
    },
]