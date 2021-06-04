<template>
  <v-navigation-drawer permanent expand-on-hover app>
    <template v-slot:img="props">
      <v-img
        :gradient="`to bottom, rgba(0, 0, 0, .8), rgba(0, 0, 0, .8)`"
        v-bind="props"
      />
    </template>

    <v-list dense nav>
      <v-list-item>
        <v-list-item-avatar
          class="align-self-center"
          color="red darken-3"
          contain
          rounded
          size="24"
          >V</v-list-item-avatar
        >

        <v-list-item-content>
          <v-list-item-title class="overline">Vagabond</v-list-item-title>
        </v-list-item-content>
      </v-list-item>
    </v-list>

    <v-divider class="mb-2" />

    <v-list expand nav>
      <div />
      <sidebar-item
        :item="{
          icon: 'mdi-view-dashboard',
          title: 'Overview',
          to: '/',
        }"
      />
      <!-- <sidebar-item
        :item="{
          icon: 'mdi-signal',
          title: 'LTE',
          to: '/lte',
        }"
      /> -->
      <sidebar-item
        :item="{
          icon: 'mdi-wifi',
          title: 'Wireless Client',
          to: '/wireless-client',
        }"
      />
      <!-- <sidebar-item
        :item="{
          icon: 'mdi-key-variant',
          title: 'Wireguard',
          to: '/wireguard',
        }"
      /> -->
      <!-- <sidebar-item
        :item="{
          icon: 'mdi-access-point-network',
          title: 'Access Point',
          to: '/access-point',
        }"
      /> -->
      <div />
    </v-list>

    <template v-slot:append>
      <sidebar-item
        :item="{
          title: 'Administration',
          icon: 'mdi-hammer-wrench',
          to: '/administration',
        }"
      />
    </template>
  </v-navigation-drawer>
</template>

<script>
export default {
  name: 'Navigation',

  components: {
    SidebarItem: () => import('@/views/components/widgets/SidebarItem'),
  },

  computed: {
    computedItems() {
      return this.items.map(this.mapItem)
    },
  },

  methods: {
    mapItem(item) {
      return {
        ...item,
        children: item.children ? item.children.map(this.mapItem) : undefined,
        title: item.title,
      }
    },
  },
}
</script>

<style lang="sass">
@import '~vuetify/src/styles/tools/_rtl.sass'

#core-navigation-drawer
  .v-list-group__header.v-list-item--active:before
    opacity: .24

  .v-list-item
    &__icon--text,
    &__icon:first-child
      justify-content: center
      text-align: center
      width: 20px

      +ltr()
        margin-right: 24px
        margin-left: 12px !important

      +rtl()
        margin-left: 24px
        margin-right: 12px !important

  .v-list--dense
    .v-list-item
      &__icon--text,
      &__icon:first-child
        margin-top: 10px

  .v-list-group--sub-group
    .v-list-item
      +ltr()
        padding-left: 8px

      +rtl()
        padding-right: 8px

    .v-list-group__header
      +ltr()
        padding-right: 0

      +rtl()
        padding-right: 0

      .v-list-item__icon--text
        margin-top: 19px
        order: 0

      .v-list-group__header__prepend-icon
        order: 2

        +ltr()
          margin-right: 8px

        +rtl()
          margin-left: 8px
</style>
