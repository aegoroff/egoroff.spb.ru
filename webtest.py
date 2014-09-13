# -*- coding: utf-8 -*-

__author__ = 'egr'

import unittest
from selenium import webdriver
import selenium.webdriver.chrome.service as service


class EgoroffTest(unittest.TestCase):

    def setUp(self):
        s = service.Service('D:/soft/Programming/Tools/chromedriver_win32/chromedriver.exe')
        s.start()
        capabilities = {}
        self.driver = webdriver.Remote(s.service_url, capabilities)

    def home_page_nav(self):
        driver = self.driver
        driver.get('http://localhost:8080/')
        nav = driver.find_elements_by_css_selector('nav > ul.nav.navbar-nav > li > a')
        #self.assertIn(5, nav.__len__())
        self.assertIsNotNone(nav)


    def tearDown(self):
        self.driver.close()


if __name__ == '__main__':
    unittest.main()