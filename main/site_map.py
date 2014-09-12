# -*- coding: utf-8 -*-

__author__ = 'egr'

ID = "id"
CLASS = "class"
TITLE = "title"
CHILDS = "childs"

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