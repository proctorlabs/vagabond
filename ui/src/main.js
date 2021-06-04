import Vue from 'vue'
import vuetify from '@/plugins/vuetify'
import App from './App.vue'
import router from './router'
import store from './store'
import VueNativeSock from 'vue-native-websocket'
import './sass/main.scss'

var host = window.location.host

Vue.use(VueNativeSock, 'ws://' + host + '/api/sock', {
  store: store,
  reconnection: true,
  reconnectionDelay: 6000,
})

Vue.config.productionTip = false

new Vue({
  router,
  store,
  vuetify,
  render: (h) => h(App),
}).$mount('#app')
