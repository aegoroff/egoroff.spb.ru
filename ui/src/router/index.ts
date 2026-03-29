import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/posts/:page?',
    name: 'Блог',
    component: () => import(/* webpackChunkName: "posts" */ '../views/admin/PostsView.vue')
  },
  {
    path: '/downloads/:page?',
    name: 'Загрузки',
    component: () => import(/* webpackChunkName: "users" */ '../views/admin/DownloadsView.vue')
  },
  {
    path: '/users',
    name: 'Пользователи',
    component: () => import(/* webpackChunkName: "users" */ '../views/admin/UsersView.vue')
  }
]

export function createAdminRouter() {
  return createRouter({
    history: createWebHashHistory(),
    routes
  })
}
