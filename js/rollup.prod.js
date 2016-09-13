import replace from 'rollup-plugin-replace';
import uglify from 'rollup-plugin-uglify';

import config from './rollup.dev';

config.dest = 'dist/app.min.js';
config.plugins[3] = replace({
  'process.env.NODE_ENV': JSON.stringify('production')
});
config.plugins.push(uglify());

export default config;
