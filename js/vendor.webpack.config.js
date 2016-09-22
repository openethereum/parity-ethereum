var webpack = require('webpack');

var ENV = process.env.NODE_ENV || 'development';
var isProd = ENV === 'production';

module.exports = {
  entry: {
    vendor: [
      'react', 'react-dom', 'react-redux', 'react-router',
      'redux', 'redux-thunk', 'react-router-redux',
      'lodash', 'material-ui', 'blockies'
    ]
  },
  output: {
    filename: 'vendor.bundle.js',
    path: 'build/',
    library: 'vendor_lib'
  },
  plugins: (function () {
    var plugins = [
      new webpack.DllPlugin({
        name: 'vendor_lib',
        path: 'build/vendor-manifest.json'
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
