#!/usr/bin/python2.7
# coding=UTF-8
#from datetime import datetime
import util
import datetime

__author__ = 'egr'

import unittest


class SiteMapTest(unittest.TestCase):
    def testSingle(self):
        pages = [
            {
                "loc" : "http://www.egoroff.spb.ru/",
                "changefreq" : "monthly",
                "priority" : "1.0"
            }
        ]
        x = util.create_site_map_xml(pages)
        self.assertEqual('<?xml version=\'1.0\' encoding=\'UTF-8\'?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>http://www.egoroff.spb.ru/</loc><changefreq>monthly</changefreq><priority>1.0</priority></url></urlset>', x)


class HumanSizeFormatTest(unittest.TestCase):

    def testNow(self):
        r = util.format_datetime_ago(datetime.datetime.utcnow())
        self.assertEqual(u'0 секунд назад', r)

    def testMinuteAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=1)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'минуту назад', r)

    def test2MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=2)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'2 минуты назад', r)

    def test3MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=3)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'3 минуты назад', r)

    def test4MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=4)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'4 минуты назад', r)

    def test5MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=5)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'5 минут назад', r)

    def test11MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=11)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'11 минут назад', r)

    def test12MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=12)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'12 минут назад', r)

    def test13MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=13)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'13 минут назад', r)

    def test14MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=14)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'14 минут назад', r)

    def test21MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=21)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'21 минуту назад', r)

    def test22MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=22)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'22 минуты назад', r)

    def test24MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=24)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'24 минуты назад', r)

    def test25MinutesAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(minutes=25)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'25 минут назад', r)

    def testHourAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(hours=1)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'час назад', r)

    def test2HoursAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(hours=2)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'2 часа назад', r)

    def test5HoursAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(hours=5)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'5 часов назад', r)

    def test21HoursAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(hours=21)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'21 час назад', r)

    def test24HoursAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(hours=24)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'вчера', r)

    def test25HoursAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(hours=25)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'вчера', r)

    def test47HoursAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(hours=47)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'вчера', r)

    def test48HoursAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(hours=48)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'2 дня назад', r)

    def test5DaysAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(days=5)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'5 дней назад', r)

    def test21DaysAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(days=21)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'21 день назад', r)

    def test1MonthAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(days=32)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'1 месяц назад', r)

    def test2MonthAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(days=65)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'2 месяца назад', r)

    def test5MonthAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(days=155)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'5 месяцев назад', r)

    def test1YearAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(days=365)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'1 год назад', r)

    def test2YearAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(days=730)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'2 года назад', r)

    def test5YearAgo(self):
        d = datetime.datetime.utcnow() - datetime.timedelta(days=1825)
        r = util.format_datetime_ago(d)
        self.assertEqual(u'5 лет назад', r)


if __name__ == '__main__':
    unittest.main()