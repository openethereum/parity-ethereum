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
const path = require('path');
const fs = require('fs');

const WebpackConfig = require('./app');

const compiler = webpack(WebpackConfig);

compiler.run(function handler (err, stats) {
  if (err) {
    return console.error(err);
  }

  // @see https://github.com/webpack/webpack/blob/324d309107f00cfc38ec727521563d309339b2ec/lib/Stats.js#L790
  // Accepted values: none, errors-only, minimal, normal, verbose
  const options = WebpackStats.presetToOptions('normal');

  options.timings = true;

  const output = stats.toString(options);
  const assets = Object.keys(stats.compilation.assets);

  const embedPath = path.resolve(WebpackConfig.output.path, './embed.json');
  const embedData = { assets: assets };

  fs.writeFileSync(embedPath, JSON.stringify(embedData, null, 2));

  process.stdout.write('\n');
  process.stdout.write(output);
  process.stdout.write('\n\n');
});
