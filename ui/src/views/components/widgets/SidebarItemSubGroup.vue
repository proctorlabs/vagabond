<template>
  <sidebar-item-group :item="item" text sub-group />
</template>

<script>
export default {
  name: 'ItemSubGroup',

  components: {
    SidebarItemGroup: () =>
      import('@/views/components/widgets/SidebarItemGroup'),
  },

  props: {
    item: {
      type: Object,
      default: () => ({
        avatar: undefined,
        group: undefined,
        title: undefined,
        children: [],
      }),
    },
  },
}
</script>
