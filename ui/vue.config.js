/**
 * @type {import('@vue/cli-service').ProjectOptions}
 */
module.exports = {
  lintOnSave: false,
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
  }
}
