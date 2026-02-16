import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import BugDetail from '@/views/BugDetail.vue'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/Home.vue')
  },
  {
    path: '/bug/:id',
    name: 'bug-detail',
    component: BugDetail,
    props: true
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
