module.exports = function(rpc) {
  return {
    getCoins: function() {
      return rpc.get('getcoins');
    },

    getMarketInfo: function(pair) {
      return rpc.get(`marketinfo/${pair}`);
    },

    getStatus: function(depositAddress) {
      return rpc.get(`txStat/${depositAddress}`);
    },

    shift: function(toAddress, returnAddress, pair) {
      return rpc.post('shift', {
        withdrawal: toAddress,
        pair: pair,
        returnAddress: returnAddress
      });
    }
  };
};
