import Vue from 'vue'
import { BootstrapVue, IconsPlugin } from 'bootstrap-vue/src'
import App from './App.vue'
import { library } from '@fortawesome/fontawesome-svg-core'
import { faBook, faBriefcase, faSearch, faHome, faUser } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import './App.scss'

Vue.config.productionTip = false
Vue.use(BootstrapVue)
Vue.use(IconsPlugin)

library.add(faBook, faBriefcase, faSearch, faHome, faUser)
Vue.component('font-awesome-icon', FontAwesomeIcon)

new Vue({
  render: h => h(App)
}).$mount('#app')
