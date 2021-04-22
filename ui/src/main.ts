import Vue from 'vue'
import VueTypeScriptInject from 'vue-typescript-inject'
import VueHighlightJS from 'vue-highlight.js'
import VueSocialSharing from 'vue-social-sharing'
import { BootstrapVue, IconsPlugin } from 'bootstrap-vue/src'
import App from './App.vue'
import { library } from '@fortawesome/fontawesome-svg-core'
import {
  faBook,
  faBriefcase,
  faSearch,
  faHome,
  faUser,
  faCalendarAlt,
  faDownload,
  faSignInAlt,
  faSignOutAlt
} from '@fortawesome/free-solid-svg-icons'
import { faGoogle, faFacebook, faGithub, faVk, faTwitter } from '@fortawesome/free-brands-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import VueProgressBar from 'vue-progressbar'
import AppIcon from './components/AppIcon.vue'
import DateFormatter from '@/components/DateFomatter.vue'
import FromNow from '@/components/FromNow.vue'
import BlogNavigation from '@/components/BlogNavigation.vue'

import './App.scss'
import BlogAnnounces from '@/components/BlogAnnounces.vue'
import BlogTitle from '@/components/BlogTitle.vue'
import Search from '@/components/Search.vue'
import Profile from '@/components/Profile.vue'
import Social from '@/components/Social.vue'

// Highlight.js languages (All languages)
import 'vue-highlight.js/lib/allLanguages'

/*
* Import Highlight.js theme
* Find more: https://highlightjs.org/static/demo/
*/
import 'highlight.js/styles/googlecode.css'
import Highlighter from '@/components/Highlighter.vue'

Vue.config.productionTip = false
Vue.use(VueTypeScriptInject)
Vue.use(BootstrapVue)
Vue.use(IconsPlugin)
Vue.use(VueProgressBar)
Vue.use(VueSocialSharing)
Vue.use(VueHighlightJS)

library.add(faBook, faBriefcase, faSearch, faHome, faUser, faCalendarAlt, faDownload, faSignInAlt, faSignOutAlt)
library.add(faGoogle, faFacebook, faGithub, faVk, faTwitter)
Vue.component('font-awesome-icon', FontAwesomeIcon)

export const bus = new Vue()

const app = document.getElementById('app')

if (app) {
  const t = app.getAttribute('datafld')
  new Vue({
    el: '#app',
    render: h => h(
      App,
      {
        props: {
          title: t
        }
      })
  }).$mount()
}

if (document.getElementById('blogNavigation')) {
  const bn = new BlogNavigation()
  bn.$mount('#blogNavigation')
}

const social = document.getElementById('social')
if (social) {
  const title = social.attributes.getNamedItem('property')
  const s = new Social({
    propsData: {
      title: title === null ? '' : title.value,
      url: window.location.href
    }
  })
  s.$mount('#social')
}

const siteSearch = document.getElementById('siteSearch')

if (siteSearch) {
  const key = siteSearch.attributes.getNamedItem('property')
  const cx = siteSearch.attributes.getNamedItem('datafld')
  const urlParams = new URLSearchParams(window.location.search)
  const q = urlParams.get('q')

  const s = new Search({
    propsData: {
      key: key === null ? '' : key.value,
      cx: cx === null ? '' : cx.value,
      query: q
    }
  })
  s.$mount('#siteSearch')
}

const userProfile = document.getElementById('userProfile')

if (userProfile) {
  const s = new Profile()
  s.$mount('#userProfile')
}

if (document.getElementById('blogcontainer') && window.location.hash) {
  const h = window.location.hash.substr(1)
  const ba = new BlogAnnounces({
    propsData: {
      q: h
    }
  })
  ba.$mount('#blogcontainer')

  const e = h.split('=')
  const bt = new BlogTitle({
    propsData: {
      text: 'все посты по метке: ' + e[1]
    }
  })
  bt.$mount('#blogSmallTitle')
}

const icons = document.querySelectorAll('i.icon[data-label]')
icons.forEach(x => {
  const label = x.attributes.getNamedItem('data-label')
  const type = x.attributes.getNamedItem('datatype')
  const icon = label === null ? '' : label.value
  const lib = type === null ? '' : type.value
  const appIcon = new AppIcon({
    propsData: {
      icon: icon,
      lib: lib
    }
  })
  appIcon.$mount(x)
})

const dates = document.querySelectorAll('span.date[data-label]')
dates.forEach(x => {
  const label = x.attributes.getNamedItem('data-label')
  const fmt = label === null ? 'LL' : label.value
  if (fmt === 'from-now') {
    new FromNow({
      propsData: {
        date: x.innerHTML
      }
    }).$mount(x)
  } else {
    new DateFormatter({
      propsData: {
        date: x.innerHTML,
        formatStr: fmt
      }
    }).$mount(x)
  }
})

const langMap = new Map<string, string>()
langMap.set('asm', 'x86asm')
langMap.set('hq', 'cs')
langMap.set('parser', 'parser3')
langMap.set('php', 'parser3')

function replacementLang (lang: string): string {
  const l = langMap.get(lang)
  return l !== undefined ? l : lang
}

function mountHighlighting (prefix: string, x: Element): void {
  if (!x.className.startsWith(prefix)) {
    return
  }
  const lang = x.className.replace(prefix, '')
    .replace(';', '')
  new Highlighter({
    propsData: {
      content: x.textContent,
      lang: replacementLang(lang)
    }
  }).$mount(x)
}

const snippets = document.querySelectorAll('pre, code')
snippets.forEach(x => {
  mountHighlighting('brush: ', x)
  mountHighlighting('language-', x)
})
