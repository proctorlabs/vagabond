<template>
  <v-app>
    <app-bar />

    <navigation />

    <main-view />

    <view-footer />
  </v-app>
</template>

<script>
export default {
  name: 'App',

  components: {
    AppBar: () => import('@/views/components/sections/AppBar'),
    MainView: () => import('@/views/components/sections/MainView'),
    Navigation: () => import('@/views/components/sections/Navigation'),
    ViewFooter: () => import('@/views/components/sections/ViewFooter'),
  },

  data: () => ({
    expandOnHover: false,
  }),
}
</script>
