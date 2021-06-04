import Vue from 'vue'
import VueRouter from 'vue-router'
import Dashboard from '@/views/Dashboard'
import NotFound from '@/views/NotFound.vue'
import AccessPoint from '@/views/AccessPoint.vue'
import Administration from '@/views/Administration.vue'
import LTE from '@/views/LTE.vue'
import Wifi from '@/views/Wifi.vue'

Vue.use(VueRouter)

const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: Dashboard,
  },
  {
    path: '/wireless-client',
    name: 'Wifi',
    component: Wifi,
  },
  {
    path: '/lte',
    name: 'LTE',
    component: LTE,
  },
  {
    path: '/access-point',
    name: 'AccessPoint',
    component: AccessPoint,
  },
  {
    path: '/administration',
    name: 'Administration',
    component: Administration,
  },
  {
    path: '/*',
    name: 'NotFound',
    component: NotFound,
  },
]

const router = new VueRouter({
  mode: 'history',
  base: process.env.BASE_URL,
  routes,
})

export default router
