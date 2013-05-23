# -*- coding: utf-8 -*-

from flask import Blueprint, render_template

mod = Blueprint(
    'portfolio',
    __name__,
    template_folder='templates',
    url_prefix='/portfolio'
)

@mod.route('/')
def index():
    return render_template(
        'portfolio/index.html'
    )
