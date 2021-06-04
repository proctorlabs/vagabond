<template>
  <v-dialog v-model="dialog" persistent max-width="450px">
    <template v-slot:activator="{ on, attrs }">
      <v-tooltip bottom>
        <template v-slot:activator="{ on, attrs }">
          <span v-bind="attrs" v-on="on">
            <v-btn icon :color="known ? 'light-green lighten-3' : 'cyan lighten-3'" @click="openDialog()">
              <v-icon>mdi-swap-horizontal-bold</v-icon>
            </v-btn>
          </span>
        </template>
        <span v-bind="attrs" v-on="on">Connect to {{ ssid }}</span>
      </v-tooltip>
    </template>
    <v-card>
      <v-card-title>
        <span class="text-h5">Connect to {{ ssidSelected }}</span>
      </v-card-title>
      <v-card-text>
        <v-container>
          <v-row>
            <v-col cols="12">
              <v-text-field
                label="Password*"
                type="password"
                v-model="password"
                required
              />
            </v-col>
          </v-row>
        </v-container>
        <small>*indicates required field</small>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn color="blue darken-3" text @click="closeDialog()">
          Close
        </v-btn>
        <v-btn
          color="blue lighten-1"
          text
          @click="connect(ssidSelected, password)"
        >
          Connect
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script>
import { mapActions } from 'vuex'
export default {
  name: 'Dashboard',
  props: ['ssid', 'known'],
  methods: {
    ...mapActions(['wifiConnect']),
    openDialog: function() {
      if (this.known) {
        this.wifiConnect({ ssid: this.ssid, password: '' })
      } else {
        this.dialog = true
        this.ssidSelected = this.ssid
      }
    },
    closeDialog: function() {
      this.dialog = false
      this.password = ''
    },
    connect: function(ssid, password) {
      this.wifiConnect({ ssid, password })
      this.closeDialog()
    },
  },
  data: () => ({
    dialog: false,
    password: '',
    ssidSelected: '',
  }),
}
</script>
