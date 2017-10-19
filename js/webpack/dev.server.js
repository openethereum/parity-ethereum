// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

const webpack = require('webpack');
const WebpackStats = require('webpack/lib/Stats');
const webpackDevMiddleware = require('webpack-dev-middleware');
const webpackHotMiddleware = require('webpack-hot-middleware');

const http = require('http');
const express = require('express');
const ProgressBar = require('progress');

const webpackConfig = require('./app');
const Shared = require('./shared');

let progressBar = { update: () => {} };

/**
 * Add webpack hot middleware to each entry in the config
 * and HMR to the plugins
 */
(function updateWebpackConfig () {
  webpackConfig.performance = { hints: false };

  Object.keys(webpackConfig.entry).forEach((key) => {
    const entry = webpackConfig.entry[key];

    webpackConfig.entry[key] = [].concat(
      'react-hot-loader/patch',
      'webpack-hot-middleware/client?reload=true',
      entry
    );
  });

  webpackConfig.plugins.push(new webpack.HotModuleReplacementPlugin());
  webpackConfig.plugins.push(new webpack.NamedModulesPlugin());
  webpackConfig.plugins.push(new webpack.NoEmitOnErrorsPlugin());

  webpackConfig.plugins.push(new webpack.ProgressPlugin(
    (percentage) => progressBar.update(percentage)
  ));
})();

const app = express();
const compiler = webpack(webpackConfig);

app.use(webpackHotMiddleware(compiler, {
  log: console.log
}));

app.use(webpackDevMiddleware(compiler, {
  noInfo: true,
  quiet: false,
  progress: true,
  publicPath: webpackConfig.output.publicPath,
  stats: {
    colors: true
  },
  reporter: function (data) {
    // @see https://github.com/webpack/webpack/blob/324d309107f00cfc38ec727521563d309339b2ec/lib/Stats.js#L790
    // Accepted values: none, errors-only, minimal, normal, verbose
    const options = WebpackStats.presetToOptions('minimal');

    options.timings = true;

    const output = data.stats.toString(options);

    process.stdout.write('\n');
    process.stdout.write(output);
    process.stdout.write('\n\n');
  }
}));

// Add the dev proxies in the express App
Shared.addProxies(app);

app.use(express.static(webpackConfig.output.path));

const server = http.createServer(app);

server.listen(process.env.PORT || 3000, function () {
  console.log('Listening on port', server.address().port);
  progressBar = new ProgressBar('[:bar] :percent :etas', { total: 50 });
});
