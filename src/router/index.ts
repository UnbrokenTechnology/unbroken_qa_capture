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
  },
  {
    path: '/session-review',
    name: 'session-review',
    component: () => import('@/views/SessionReview.vue')
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
