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
                  Up
                </th>
                <th class="text-left">
                  Local Addresses
                </th>
              </tr>
            </thead>

            <tbody>
              <tr v-for="item in interfaces" :key="item.name">
                <td>
                  {{ item.name }}
                </td>
                <td>{{ item.up }}</td>
                <td>
                  <span
                    v-for="(addr, key, index) in item.addresses"
                    :key="addr.local"
                  >
                    <span v-if="addr.type == 'Mac'">
                      Mac: {{ formatMac(addr.address) }}
                    </span>
                    <span v-else-if="addr.type == 'Ipv4'"
                      >IP4: {{ addr.address }}</span
                    >
                    <span v-else-if="addr.type == 'Ipv6'"
                      >IP6: {{ addr.address }}</span
                    >
                    <span v-else>{{ addr.type }}: {{ addr.address }}</span>
                    <br
                      v-if="index != Object.keys(item.addresses).length - 1"
                    />
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
              <tr v-for="(service, name) in services" :key="name">
                <td>{{ name }}</td>
                <td>{{ service.state }}</td>
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
    formatMac: function(mac) {
      var result = []
      mac.forEach((u8) => {
        var unit = u8.toString(16).toUpperCase()
        if (unit.length < 2) {
          unit = '0' + unit
        }
        result.push(unit)
      })
      return result.join(':')
    },
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
