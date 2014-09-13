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

    def test_top_nav_count(self):
        driver = self.driver
        driver.get('http://localhost:8080/')
        nav = driver.find_elements_by_css_selector('nav > ul.nav.navbar-nav > li > a')
        self.assertEquals(6, nav.__len__())

    def test_portfolio_click(self):
        driver = self.driver
        driver.get('http://localhost:8080/')
        a = driver.find_element_by_css_selector('body > header > div > nav > ul:nth-child(1) > li:nth-child(1) > a')
        a.click()

    def test_blog_click(self):
        driver = self.driver
        driver.get('http://localhost:8080/')
        a = driver.find_element_by_css_selector('body > header > div > nav > ul:nth-child(1) > li:nth-child(2) > a')
        a.click()

    def test_search_click(self):
        driver = self.driver
        driver.get('http://localhost:8080/')
        a = driver.find_element_by_css_selector('body > header > div > nav > ul:nth-child(1) > li:nth-child(3) > a')
        a.click()

    def test_rss_click(self):
        driver = self.driver
        driver.get('http://localhost:8080/')
        a = driver.find_element_by_css_selector('body > header > div > nav > ul:nth-child(1) > li:nth-child(4) > a')
        a.click()

    def test_feedback_click(self):
        driver = self.driver
        driver.get('http://localhost:8080/')
        a = driver.find_element_by_css_selector('body > header > div > nav > ul:nth-child(1) > li:nth-child(5) > a')
        a.click()

    def tearDown(self):
        self.driver.close()


if __name__ == '__main__':
    unittest.main()