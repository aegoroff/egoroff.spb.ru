/**
 * @type {import('@vue/cli-service').ProjectOptions}
 */
module.exports = {
  lintOnSave: false,
  parallel: false,
  transpileDependencies: true,
  outputDir: '../static/dist/',
  productionSourceMap: false,
  configureWebpack: {
    module: {
      rules: [
        {
          test: /\.tsx?$/,
          loader: 'esbuild-loader',
          options: {
            loader: 'ts',
            target: 'es2020'
          }
        }
      ]
    }
  },
  chainWebpack: config => {
    // Disable the broken fork-ts-checker plugin
    config.plugins.delete('fork-ts-checker');
  }
}
