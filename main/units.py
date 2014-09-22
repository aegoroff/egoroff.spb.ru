# -*- coding: utf-8 -*-

__author__ = 'egorov'

INT64_BITS_COUNT = 64
BINARY_THOUSAND = 1024

UNITS = [
    u"байт",
    u"Кб",
    u"Мб",
    u"Гб",
    u"Тб",
    u"Пб",
    u"Эб",
    u"Зб",
    u"Йб"
]


def ilog(x):
    """Calculates integer logarithm

    Args:
        x: int, the number to calculate logarithm of.
    """
    n = INT64_BITS_COUNT
    c = INT64_BITS_COUNT / 2
    while True:
        y = x >> c
        if y:
            n -= c
            x = y
        c >>= 1
        if not c:
            break
    n -= x >> (INT64_BITS_COUNT - 1)
    return (INT64_BITS_COUNT - 1) - (n - x)


def normalize(bytesValue):
    if not bytesValue:
        return 0, bytesValue
    units = ilog(bytesValue) / ilog(BINARY_THOUSAND)
    if not units:
        value = bytesValue
    else:
        value = float(bytesValue) / pow(BINARY_THOUSAND, units)
    return units, value


def formatToHumanSize(bytesValue, precision=2):
    units, value = normalize(bytesValue)
    f = u'{0:.{2}f} {1}'
    if not units:
        f = u'{0} {1}'
    return f.format(value, UNITS[units], precision)