# -*- coding: utf-8 -*-
from flask import jsonify
from functools import wraps
import logging

def jsonify_success(obj, success=True, status=200):
    res = jsonify({'success': success, 'result': obj})
    res.status_code = status
    return res

def except_wrap(func):
    @wraps(func)
    def wrapped(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except Exception, e:
            res = {
                'function': func.__name__,
                'msg': str(e),
            }
            logging.error(res)
            if hasattr(e, 'status'):
                status = getattr(e, 'status')
                if status:
                    return jsonify_success(
                        res, success=False,
                        status=getattr(e, 'status')
                    )
            return jsonify_success(res, success=False)
    return wrapped

class ApiException(Exception):
    def __init__(self, message, status = 400):
        Exception.__init__(self, message)
        self.status = status
