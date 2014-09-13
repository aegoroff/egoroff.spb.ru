# -*- coding: utf-8 -*-

__author__ = 'egr'

import unittest
from selenium import webdriver
import selenium.webdriver.chrome.service as service
import time


class EgoroffTest(unittest.TestCase):

    def setUp(self):
        s = service.Service('D:/soft/Programming/Tools/chromedriver_win32/chromedriver.exe')
        s.start()
        capabilities = {}
        self.driver = webdriver.Remote(s.service_url, capabilities)

    def test_top_nav_count(self):
        driver = self.driver
        driver.get('http://localhost:8080/')
        nav = driver.find_elements_by_css_selector('nav > ul.nav.navbar-nav > li > a')
        self.assertEquals(6, nav.__len__())

    def test_blog_tags(self):
        driver = self.driver
        driver.get('http://localhost:8080/blog/')
        tags = driver.find_elements_by_css_selector('div.tags > ul > li > a')
        for tag in tags:
            tag.click()
            time.sleep(1)
            links = driver.find_elements_by_css_selector('#log > dt > a')
            self.assertTrue(links.__len__() > 0)

    def test_blog_archive(self):
        driver = self.driver
        driver.get('http://localhost:8080/blog/')
        years = driver.find_elements_by_css_selector('div#accordion > div.list-group')
        for year in years:
            months = year.find_elements_by_css_selector('a.list-group-item')
            for month in months:
                if month.is_displayed():
                    month.click()
                    time.sleep(1)
                    links = driver.find_elements_by_css_selector('#log > dt > a')
                    self.assertTrue(links.__len__() > 0)



    def tearDown(self):
        self.driver.close()


if __name__ == '__main__':
    unittest.main()