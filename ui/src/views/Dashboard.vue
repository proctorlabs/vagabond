<template>
  <v-sheet
    elevation="4"
    class="pl-6 pb-6 pr-6 pt-3 col-12 rounded-xl fill-height"
    rounded
    outlined
    app
  >
    <v-subheader>Overview</v-subheader>
    <h2 class="text-center">Services online: {{ online }}</h2>
    <h3 class="text-right"><v-btn v-on:click="getStatus">update</v-btn></h3>
  </v-sheet>
</template>

<script>
import { mapGetters, mapActions } from 'vuex'
export default {
  name: 'Dashboard',
  methods: {
    ...mapActions(['getStatus']),
  },
  computed: {
    ...mapGetters({ online: 'serviceOnline' }),
  },
  mounted: function() {
    this.getStatus()
  },
}
</script>
