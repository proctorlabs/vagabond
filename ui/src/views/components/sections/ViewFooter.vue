<template>
  <v-footer id="view-footer">
    <v-container>
      <v-row align="center" no-gutters>
        <v-col cols="12" md="auto" v-if="socketOnline">
          <div class="body-1 font-weight-light pt-6 pt-md-0 text-center">
            API Connected
          </div>
        </v-col>

        <v-col cols="12" md="auto" v-else>
          <div class="body-1 font-weight-light pt-6 pt-md-0 text-center">
            API Not Connected!
          </div>
        </v-col>

        <v-spacer class="hidden-sm-and-down" />

        <v-col cols="12" md="auto">
          <div class="body-1 font-weight-light pt-6 pt-md-0 text-center">
            &copy; 2021
          </div>
        </v-col>
      </v-row>
    </v-container>
  </v-footer>
</template>

<script>
import { mapGetters } from 'vuex'
export default {
  name: 'ViewFooter',
  computed: {
    ...mapGetters({ socketOnline: 'socketOnline' }),
  },
}
</script>

<style lang="sass">
#view-footer
  a
    font-size: .825rem
    font-weight: 500
    text-decoration: none
    text-transform: uppercase
</style>
