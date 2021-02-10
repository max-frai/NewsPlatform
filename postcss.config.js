const purgecss = require('@fullhuman/postcss-purgecss')
const cssnano = require('cssnano')

module.exports = {
  plugins: [
    require('tailwindcss'),
    // require('autoprefixer'),
    cssnano({
      preset: 'default'
    }),
    purgecss({
      content: ['./news_templates/**/*.tera', './news_templates/*.tera', './news_templates/modules/**/*.tera', './news_templates/media/*.tera', './news_templates/parts/*.tera', './news_svelte/src/**/*.svelte', './news_svelte/src/*.svelte'],
      whitelistPatterns: [/svelte-/],
      defaultExtractor: content => {
         const regExp = new RegExp(/[A-Za-z0-9-_:/]+/g);
         const matchedTokens = [];
         let match = regExp.exec(content);

        // console.log(content);

         while (match) {
          //  console.log(match[0]);
            if (match[0].startsWith('class:'))
               matchedTokens.push(match[0].substring(6));
            else
               matchedTokens.push(match[0]);
            match = regExp.exec(content);
         }

        // console.log(matchedTokens);
         return matchedTokens;
      }
    })
  ]
}
