/** @type {import('tailwindcss').Config} */

export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx,vue}",
    "./node_modules/flowbite/**/*.js",
    'node_modules/flowbite-vue/**/*.{js,jsx,ts,tsx,vue}',
  ],
  theme: {
    extend: {},
  },
  plugins: [
    require('flowbite/plugin')
  ],

  // This is needed to prevent Tailwind from overriding existing styles
  // https://stackoverflow.com/questions/72654538/tailwind-css-breaking-existing-styles
  // TODO: fix the screens to look good with preflight enabled
  corePlugins: {
    preflight: false,
  }
}

