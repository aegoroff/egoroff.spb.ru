# -*- coding: utf-8 -*-

import util
import flask
from flask import Blueprint

__author__ = 'egorov'


mod = Blueprint(
    'compatibility',
    __name__
)


# Redirection rules for the old site materials

@mod.route('/opinions/')
def opinions():
    return flask.redirect(flask.url_for('news.index'), code=301)

@mod.route('/opinions/<int:key_id>.html')
def opinions_files(key_id):
    remapping = {
        1: 25002,
        4: 31001,
        8: 6003,
        11: 30001,
        13: 3006,
        18: 29001,
        21: 9002,
        22: 2004,
        24: 25003,
        25: 22002,
        26: 27002,
        27: 27001,
        28: 14004,
        29: 8003,
        30: 6004
    }
    return util.redirect(key_id, remapping)

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

