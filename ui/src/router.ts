import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import MainLayout from './layouts/MainLayout.vue'
import HomeView from './views/HomeView.vue'
import ReviewView from './views/ReviewView.vue'

// Hash history: the tray window loads index.html#/tray.
// Screens other than Home load on first visit, so startup stays fast.
export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: MainLayout,
    children: [
      { path: '', component: HomeView },
      { path: 'review', component: ReviewView },
      { path: 'apps', component: () => import('./views/AppsView.vue') },
      { path: 'performance', component: () => import('./views/PerformanceView.vue') },
      { path: 'space', component: () => import('./views/SpaceView.vue') },
      { path: 'settings', component: () => import('./views/SettingsView.vue') },
      { path: 'about', component: () => import('./views/AboutView.vue') }
    ]
  },
  { path: '/tray', component: () => import('./views/TrayView.vue') },
  { path: '/:pathMatch(.*)*', redirect: '/' }
]

export const router = createRouter({ history: createWebHashHistory(), routes })
