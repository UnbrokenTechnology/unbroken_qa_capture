import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import BugDetail from '@/views/BugDetail.vue'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/IdleView.vue')
  },
  {
    path: '/active-session',
    name: 'active-session',
    component: () => import('@/views/ActiveSessionView.vue')
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
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/views/Settings.vue')
  },
  {
    path: '/annotate',
    name: 'annotate',
    component: () => import('@/views/AnnotateView.vue')
  },
  {
    path: '/session-notes',
    name: 'session-notes',
    component: () => import('@/views/SessionNotesView.vue')
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
