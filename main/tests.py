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



if __name__ == '__main__':
    unittest.main()