import Vue from 'vue'
import VueRouter, { RouteConfig } from 'vue-router'

Vue.use(VueRouter)

const routes: Array<RouteConfig> = [
  {
    path: '/posts/:page?',
    name: 'Блог',
    // route level code-splitting
    // this generates a separate chunk (posts.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: () => import(/* webpackChunkName: "posts" */ '../views/admin/Posts.vue')
  },
  {
    path: '/downloads/:page?',
    name: 'Загрузки',
    // route level code-splitting
    // this generates a separate chunk (downloads.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: () => import(/* webpackChunkName: "users" */ '../views/admin/Downloads.vue')
  },
  {
    path: '/users',
    name: 'Пользователи',
    // route level code-splitting
    // this generates a separate chunk (users.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: () => import(/* webpackChunkName: "users" */ '../views/admin/Users.vue')
  }
]

export default routes
