# -*- coding: utf-8 -*-

import flask
from flask import request

__author__ = 'egr'

ID = "id"
CLASS = "class"
TITLE = "title"
CHILDS = "childs"
DESCR = "description"


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
    current_title = None
    meta_description = None
    if curr:
        current_id = curr[ID]
    if root:
        root_id = root[ID]
        if DESCR in root:
            meta_description = root[DESCR]

    for s in MAP:
        if s[ID] == root_id:
            sections = s[CHILDS]
            break

    if request.path != flask.url_for('welcome') and request.path != flask.url_for('admin_config_update'):
        start = [(root[ID], root[TITLE], root[CLASS])]
        if curr:
            current_title = curr[TITLE]
            if DESCR in curr:
                meta_description = curr[DESCR]
            else:
                meta_description = None  # reset welcome meta in no section description defined
            if request.path == uri and (not request.query_string or request.query_string == ''):
                breadcrumbs = create_breadcrumbs(start, [])
            else:
                breadcrumbs = create_breadcrumbs(start, [curr])
                meta_description = None  # reset meta for section root
        else:
            breadcrumbs = create_breadcrumbs(start, [])
    return dict(
        current_id=current_id,
        root_id=root_id,
        breadcrumbs=breadcrumbs,
        sections=sections,
        current_section=curr,
        current_title=current_title,
        meta_description=meta_description)

MAP = [
    {
        ID: "welcome",
        CLASS: "fa fa-home",
        TITLE: u'Главная',
        DESCR: u'Сайт об обычном программировании и веб технологиях, вроде apache, парсера и других. Есть инструменты для вычисления хэшей и восстановления строк',
        CHILDS : [
            {
                ID: "portfolio.index",
                CLASS: "fa fa-briefcase",
                TITLE: u"Портфель",
                DESCR: u'Из портфеля можно загрузить разные полезные вещи. Также тут есть переводы документации Apache'
            },
            {
                ID: "news.index",
                CLASS: "fa fa-book",
                TITLE: u"Блог",
                DESCR: u'Блог. тут я пишу на разные айтишные темы.'
            },
            {
                ID: "search",
                CLASS: "fa fa-search",
                TITLE: u"Поиск",
                DESCR: u'Поиск по сайту с использованием поиска Google. Применяется асинхронный java script AJAX'
            },
            {
                ID: "feedback",
                CLASS: "fa fa-comment",
                TITLE: u"Фидбек",
                DESCR: u'Форма для обратной связи со мной.'
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
                CLASS: "fa fa-fw fa-edit",
                TITLE: u"Блог"
            },
            {
                ID: "admin.file.index",
                CLASS: "fa fa-archive",
                TITLE: u"Файловый менеджер"
            },
            {
                ID: "user_list",
                CLASS: "fa fa-fw fa-group",
                TITLE: u"Список пользователей"
            },
        ]
    },
]