<template>
  <v-sheet
    elevation="4"
    class="pl-6 pb-6 pr-6 pt-3 col-12 rounded-xl fill-height"
    rounded
    outlined
    app
  >
    <h1 class="text-center">Overview</h1>
    <v-row no-gutters>
      <v-spacer />
      <v-col cols="5">
        <v-simple-table dense>
          <template v-slot:default>
            <thead>
              <tr>
                <th class="text-left">
                  Interface
                </th>
                <th class="text-left">
                  MAC
                </th>
                <th class="text-left">
                  Local Addresses
                </th>
              </tr>
            </thead>

            <tbody>
              <tr v-for="item in interfaces" :key="item.ifname">
                <td>
                  {{ item.ifname }}
                </td>
                <td>{{ item.address }}</td>
                <td>
                  <span v-for="addr in item.addr_info" :key="addr.local"
                    >{{ addr.local }} |
                  </span>
                </td>
              </tr>
            </tbody>
          </template>
        </v-simple-table>
      </v-col>

      <v-spacer />

      <v-col cols="5">
        <v-simple-table dense>
          <template v-slot:default>
            <tbody>
              <tr>
                <td>hostapd</td>
                <td>{{ services.hostapd.running ? 'running' : 'stopped' }}</td>
              </tr>
              <tr>
                <td>dhcpd:</td>
                <td>{{ services.dhcpd.running ? 'running' : 'stopped' }}</td>
              </tr>
              <tr>
                <td>unbound</td>
                <td>{{ services.unbound.running ? 'running' : 'stopped' }}</td>
              </tr>
            </tbody>
          </template>
        </v-simple-table>
      </v-col>

      <v-spacer />
    </v-row>
  </v-sheet>
</template>

<script>
import { mapGetters, mapActions } from 'vuex'
export default {
  name: 'Dashboard',
  methods: {
    ...mapActions(['getStatus', 'listInterfaces']),
  },
  computed: {
    ...mapGetters({ interfaces: 'interfaces', services: 'services' }),
  },
  mounted: function() {
    this.listInterfaces()
    this.getStatus()
  },
}
</script>
