import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'home', component: () => import('../views/Home.vue') },
    { path: '/report/:id', name: 'report', component: () => import('../views/Report.vue'), props: true },
    { path: '/table/:tableName', name: 'table', component: () => import('../views/Report.vue'), props: true }
  ]
})

export default router
