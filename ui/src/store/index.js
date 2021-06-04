import Vue from 'vue'
import Vuex from 'vuex'
import handler from './socket_handler'

Vue.use(Vuex)

var store_obj = {
  state: {
    interfaces: {},
    services: {},
    service: {
      status: null,
      hostapd: {
        log: [],
      },
      dhcpd: {
        log: [],
      },
      unbound: {
        log: [],
      },
    },
    socket: {
      isConnected: false,
      reconnectError: false,
    },
    wifiScan: [],
    wifiStatus: {},
  },

  getters: {
    serviceOnline: (state) => {
      return state.status != null
    },
    services: (state) => {
      return state.services
    },
    socketOnline: (state) => {
      return state.socket.isConnected
    },
    socketReconnectFailed: (state) => {
      return state.socket.reconnectError
    },
    interfaces: (state) => {
      return state.interfaces
    },
    wifiNetworks: (state) => {
      return state.wifiScan
    },
    wifiStatus: (state) => {
      return state.wifiStatus
    },
  },

  mutations: {
    SOCKET_ONOPEN: (state, event) => {
      Vue.prototype.$socket = event.currentTarget
      state.socket.isConnected = true
    },

    SOCKET_ONCLOSE: (state) => {
      state.socket.isConnected = false
    },

    SOCKET_ONERROR: (state, event) => {
      console.error(state, event)
    },

    SOCKET_ONMESSAGE: (state, message) => {
      var msg = JSON.parse(message.data)
      var handler_name = msg.type + '_handler'
      var method = handler[handler_name]
      if (!method) {
        console.log('No handler for message type:', msg.type)
      } else {
        method(msg.data, state)
      }
    },

    SOCKET_RECONNECT: (state, count) => {
      console.info(state, count)
      state.socket.reconnectError = false
    },

    SOCKET_RECONNECT_ERROR: (state) => {
      state.socket.reconnectError = true
    },
  },

  actions: {
    getStatus: () => {
      Vue.prototype.$socket.send(
        JSON.stringify({
          type: 'get_status',
        })
      )
    },

    listInterfaces: () => {
      Vue.prototype.$socket.send(
        JSON.stringify({
          type: 'list_interfaces',
        })
      )
    },

    wifiScan: () => {
      Vue.prototype.$socket.send(
        JSON.stringify({
          type: 'wifi_scan',
        })
      )
    },

    wifiStatus: () => {
      Vue.prototype.$socket.send(
        JSON.stringify({
          type: 'wifi_status',
        })
      )
    },

    wifiConnect: (state, { ssid, password }) => {
      Vue.prototype.$socket.send(
        JSON.stringify({
          type: 'wifi_connect',
          data: {
            ssid: ssid,
            psk: password,
          },
        })
      )
    },

    wifiDisconnect: () => {
      Vue.prototype.$socket.send(
        JSON.stringify({
          type: 'wifi_disconnect',
        })
      )
    },
  },
}

export default new Vuex.Store(store_obj)
