module.exports = function(apikey) {
  const rpc = require('./lib/rpc')(apikey);

  return require('./lib/shapeshift')(rpc);
};
