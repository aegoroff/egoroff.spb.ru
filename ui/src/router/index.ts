import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/posts/:page?',
    name: 'Блог',
    component: () => import(/* webpackChunkName: "posts" */ '../views/admin/Posts.vue')
  },
  {
    path: '/downloads/:page?',
    name: 'Загрузки',
    component: () => import(/* webpackChunkName: "users" */ '../views/admin/Downloads.vue')
  },
  {
    path: '/users',
    name: 'Пользователи',
    component: () => import(/* webpackChunkName: "users" */ '../views/admin/Users.vue')
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router