// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
  webpackConfig.plugins.push(new webpack.NoErrorsPlugin());

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
  noInfo: false,
  quiet: true,
  progress: true,
  publicPath: webpackConfig.output.publicPath,
  stats: {
    colors: true
  }
}));

app.use(express.static(webpackConfig.output.path));

// Add the dev proxies in the express App
Shared.addProxies(app);

const server = http.createServer(app);
server.listen(process.env.PORT || 3000, function () {
  console.log('Listening on port', server.address().port);
  progressBar = new ProgressBar('[:bar] :percent :etas', { total: 50 });
});
