const purgecss = require('@fullhuman/postcss-purgecss')
const cssnano = require('cssnano')

module.exports = {
  plugins: [
    require('tailwindcss'),
    require('autoprefixer'),
    cssnano({
      preset: 'default'
    }),
    purgecss({
      content: ['./templates/**/*.html', './templates/*.html', './templates/modules/**/*.html'],
      defaultExtractor: content => content.match(/[\w-/:]+(?<!:)/g) || []
    })
  ]
}
