const purgecss = require('@fullhuman/postcss-purgecss')
const cssnano = require('cssnano')

module.exports = {
  plugins: [
    require('tailwindcss'),
    // require('autoprefixer'),
    // cssnano({
    //   preset: 'default'
    // }),
    // purgecss({
    //   content: ['./templates/**/*.tera', './templates/*.tera', './templates/modules/**/*.tera', './templates/media/*.tera', './templates/parts/*.tera'],
    //   defaultExtractor: content => content.match(/[\w-/:]+(?<!:)/g) || []
    // })
  ]
}
