import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'
import DashboardView from '../views/admin/DashboardView.vue'
import PostsView from '../views/admin/PostsView.vue'
import DownloadsView from '../views/admin/DownloadsView.vue'
import UsersView from '../views/admin/UsersView.vue'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    name: 'Дашборд',
    component: DashboardView,
  },
  {
    path: '/posts/:page?',
    name: 'Блог',
    component: PostsView,
  },
  {
    path: '/downloads/:page?',
    name: 'Загрузки',
    component: DownloadsView,
  },
  {
    path: '/users',
    name: 'Пользователи',
    component: UsersView,
  },
]

export function createAdminRouter() {
  return createRouter({
    history: createWebHashHistory(),
    routes,
  })
}
