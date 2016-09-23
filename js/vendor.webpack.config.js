var webpack = require('webpack');

var ENV = process.env.NODE_ENV || 'development';
var isProd = ENV === 'production';

module.exports = {
  entry: {
    vendor: (function () {
      let vendors = [
        'react', 'react-dom', 'react-redux', 'react-router',
        'redux', 'redux-thunk', 'react-router-redux',
        'lodash', 'material-ui', 'blockies',
        'babel-polyfill'
      ];

      if (!isProd) {
        vendors = [].concat(vendors, [
          'webpack-dev-server/client?http://localhost:3000',
          'react-hot-loader', 'core-js', 'core-js/library',
          'moment', 'web3'
        ]);
      }

      return vendors;
    }())
  },
  module: {
    loaders: [
      {
        test: /\.json$/,
        loaders: ['json']
      }
    ]
  },
  output: {
    filename: '[name].bundle.js',
    path: 'build/',
    library: '[name]_lib'
  },
  plugins: (function () {
    var plugins = [
      new webpack.DllPlugin({
        name: '[name]_lib',
        path: 'build/[name]-manifest.json'
      })
    ];

    if (isProd) {
      plugins.push(new webpack.optimize.OccurrenceOrderPlugin(false));
      plugins.push(new webpack.optimize.DedupePlugin());
      plugins.push(new webpack.optimize.UglifyJsPlugin({
        screwIe8: true,
        compress: {
          warnings: false
        },
        output: {
          comments: false
        }
      }));
    }

    return plugins;
  }())
};
