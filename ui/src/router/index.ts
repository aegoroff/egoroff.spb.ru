import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    name: 'Дашборд',
    component: () => import('../views/admin/DashboardView.vue'),
  },
  {
    path: '/posts/:page?',
    name: 'Блог',
    component: () => import('../views/admin/PostsView.vue'),
  },
  {
    path: '/downloads/:page?',
    name: 'Загрузки',
    component: () => import('../views/admin/DownloadsView.vue'),
  },
  {
    path: '/users',
    name: 'Пользователи',
    component: () => import('../views/admin/UsersView.vue'),
  },
]

export function createAdminRouter() {
  const router = createRouter({
    history: createWebHashHistory(),
    routes,
  })
  const siteTitle = document.title
  router.afterEach((to) => {
    document.title = to.name ? `${String(to.name)} | ${siteTitle}` : siteTitle
  })
  return router
}
