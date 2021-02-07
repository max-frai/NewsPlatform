# Svelte template with Tailwind CSS

![](https://upload.wikimedia.org/wikipedia/commons/thumb/1/1b/Svelte_Logo.svg/199px-Svelte_Logo.svg.png) ![](https://seeklogo.com/images/T/tailwind-css-logo-5AD4175897-seeklogo.com.png)

Setting up Tailwind with Svelte is really simple, just install necessary dependencies:

```sh
npm i -D svelte-preprocess tailwindcss postcss autoprefixer
```

Create your Tailwind config file

```sh
npx tailwindcss init
```

Configure **svelte-preprocess** in `rollup.config.js`

```js
import svelte from 'rollup-plugin-svelte';
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import livereload from 'rollup-plugin-livereload';
import { terser } from 'rollup-plugin-terser';
import sveltePreprocess from 'svelte-preprocess'; // here

// ...

export default {
  // ...
  plugins: [
    svelte({
      compilerOptions: {
        // enable run-time checks when not in production
        dev: !production,
      },
      preprocess: sveltePreprocess({
        postcss: {
          plugins: [require('tailwindcss'), require('autoprefixer')],
        },
      }),
    }),
    // ...
  ],
  // ...
};
```

Next, import Tailwind styles in `src/App.svelte` :

```svelte
<script>
  export let name;
</script>

<style global>
  @import 'tailwindcss/base';

  @import 'tailwindcss/components';

  @import 'tailwindcss/utilities';
</style>

<main>
  <h1>Hello {name}!</h1>
  <p> Visit the <a href="https://svelte.dev/tutorial">Svelte tutorial</a> to learn how to build Svelte apps.</p>
</main>
```

Finally, configure purge in `tailwind.config.js`:

```js
module.exports = {
  purge: {
    enabled: true,
    content: ['./src/**/*.svelte'],
  },
  darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {},
  },
  variants: {
    extend: {},
  },
  plugins: [],
};
```

*- Changes can take up to 5 seconds to be reflected in the browser -*

## Get started

Install the dependencies...

```bash
cd Svelte-Tailwind
npm install
```

...then start [Rollup](https://rollupjs.org):

```bash
npm run dev
```

## Building and running in production mode

```bash
npm run build
```
