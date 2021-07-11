<template>
  <v-sheet
    elevation="4"
    class="pl-6 pb-6 pr-6 pt-3 col-12 rounded-xl fill-height"
    rounded
    outlined
    app
  >
    <v-row>
      <v-col cols="4">
        <h3 class="text-center">Connection Detail</h3>
        <hr />
        <v-simple-table dense>
          <tbody>
            <tr>
              <td class="text-right"><h4>Interface</h4></td>
              <td class="text-left">{{ status.name }}</td>
            </tr>
            <tr>
              <td class="text-right"><h4>Connected Network</h4></td>
              <td class="text-left" v-if="status.state == 'connected'">{{ status.connected_network.ssid }}</td>
              <td class="text-left" v-else> - </td>
            </tr>
            <tr>
              <td class="text-right"><h4>PHY</h4></td>
              <td class="text-left">{{ status.phy }}</td>
            </tr>
            <tr>
              <td class="text-right"><h4>Vendor</h4></td>
              <td class="text-left">{{ status.vendor }}</td>
            </tr>
            <tr>
              <td class="text-right"><h4>Model</h4></td>
              <td class="text-left">{{ status.model }}</td>
            </tr>
            <tr>
              <td class="text-right"><h4>MAC Address</h4></td>
              <td class="text-left">{{ status.address.toUpperCase() }}</td>
            </tr>
            <tr>
              <td class="text-right"><h4>Current State</h4></td>
              <td class="text-left">{{ titleize(status.state) }}</td>
            </tr>
            <tr>
              <td class="text-right"><h4>Current Mode</h4></td>
              <td class="text-left">{{ formatMode(status.mode) }}</td>
            </tr>
            <tr>
              <td class="text-right"><h4>Device Powered</h4></td>
              <td class="text-left">{{ status.powered ? 'yes' : 'no' }}</td>
            </tr>
            <tr>
              <td class="text-right"><h4>Scanning...</h4></td>
              <td class="text-left">{{ status.scanning ? 'yes' : 'no' }}</td>
            </tr>
            <tr>
              <td class="text-right"><h4>Supported Modes</h4></td>
              <td class="text-left">
                {{ formatModes(status.supported_modes) }}
              </td>
            </tr>
          </tbody>
        </v-simple-table>
        <hr />
        <h3 class="text-right">
          <v-btn v-on:click="wifiScan">Update Networks</v-btn>
          <v-btn v-on:click="wifiDisconnect">Disconnect</v-btn>
        </h3>
      </v-col>
      <v-col cols="8">
        <v-simple-table dense>
          <template v-slot:default>
            <thead>
              <tr>
                <th class="text-left">
                  SSID
                </th>
                <th class="text-right">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in networks" :key="item.name">
                <td class="text-left">{{ item.ssid }}</td>
                <td class="text-right">
                  <v-tooltip bottom>
                    <template v-slot:activator="{ on, attrs }">
                      <span v-bind="attrs" v-on="on">
                        <v-icon :color="securityColor(item.security)">
                          {{ securityIcon(item.security) }}
                        </v-icon>
                      </span>
                    </template>
                    <span>{{ item.security }}</span>
                  </v-tooltip>
                  <v-tooltip bottom>
                    <template v-slot:activator="{ on, attrs }">
                      <span v-bind="attrs" v-on="on">
                        <v-icon :color="wifiStrengthColor(item.signal)">
                          {{ wifiStrengthIcon(item.signal) }}
                        </v-icon>
                      </span>
                    </template>
                    <span>{{ item.signal }} dBm</span>
                  </v-tooltip>
                  <connect-dialog :ssid="item.ssid" :known="item.known" />
                </td>
              </tr>
            </tbody>
          </template>
        </v-simple-table>
      </v-col>
    </v-row>
  </v-sheet>
</template>

<script>
import { mapGetters, mapActions } from 'vuex'
export default {
  name: 'Wifi',
  methods: {
    ...mapActions(['wifiScan', 'wifiStatus', 'wifiDisconnect']),
    formatMode: function(mode) {
      switch (mode) {
        case 'ad_hoc':
          return 'Ad-Hoc'
        case 'station':
          return 'Station'
        case 'access_point':
          return 'Access Point'
        default:
          return mode
      }
    },
    formatModes: function(modes) {
      var result = []
      modes.forEach((mode) => {
        result.push(this.formatMode(mode))
      })
      return result.join(', ')
    },
    titleize: function(name) {
      var string_array = name.split(' ')
      string_array = string_array.map(function(str) {
        return str.charAt(0).toUpperCase() + str.slice(1);
      })
      return string_array.join(' ')
    },
    securityIcon: function(icon) {
      switch (icon) {
        case 'psk':
          return 'mdi-lock'
        case 'open':
          return 'mdi-lock-open-variant'
        case '8021x':
          return 'mdi-lock-alert'
        default:
          return 'mdi-help-circle'
      }
    },
    securityColor: function(icon) {
      switch (icon) {
        case 'psk':
          return 'green lighten-1'
        case 'open':
          return 'red lighten-1'
        case '8021x':
          return 'light-green lighten-1'
        default:
          return 'orange lighten-1'
      }
    },
    wifiStrengthIcon: function(rssi) {
      if (rssi > -60) {
        return 'mdi-wifi-strength-4'
      } else if (rssi > -70) {
        return 'mdi-wifi-strength-3'
      } else if (rssi > -80) {
        return 'mdi-wifi-strength-2'
      } else if (rssi > -90) {
        return 'mdi-wifi-strength-1'
      } else {
        return 'mdi-wifi-strength-outline'
      }
    },
    wifiStrengthColor: function(rssi) {
      if (rssi > -60) {
        return 'green lighten-1'
      } else if (rssi > -70) {
        return 'light-green lighten-1'
      } else if (rssi > -80) {
        return 'lime lighten-1'
      } else if (rssi > -90) {
        return 'orange lighten-1'
      } else {
        return 'red lighten-1'
      }
    },
  },
  components: {
    ConnectDialog: () => import('@/views/components/widgets/ConnectDialog'),
  },
  computed: {
    ...mapGetters({ networks: 'wifiNetworks', status: 'wifiStatus' }),
  },
  beforeDestroy() {
    clearInterval(this.wifiInterval)
  },
  mounted: function() {
    this.wifiScan()
    this.wifiStatus()
    this.wifiInterval = setInterval(() => {
      this.wifiScan()
      this.wifiStatus()
    }, 1000)
  },
}
</script>
