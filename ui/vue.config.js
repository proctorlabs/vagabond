// vue.config.js

/**
 * @type {import('@vue/cli-service').ProjectOptions}
 */
module.exports = {
  filenameHashing: false,
  transpileDependencies: ['vuetify'],
  devServer: {
    proxy: {
      '^/api.*': {
        target: 'http://127.0.0.1:5000/',
        ws: true,
        changeOrigin: true,
      },
    },
  },
}
