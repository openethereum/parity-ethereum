export default function web3extensions (web3) {
  const { Method, formatters } = web3._extend;

  return [{
    property: 'personal',
    methods: [
      new Method({
        name: 'signAndSendTransaction',
        call: 'personal_signAndSendTransaction',
        params: 2,
        inputFormatter: [formatters.inputTransactionFormatter, null]
      }),
      new Method({
        name: 'signerEnabled',
        call: 'personal_signerEnabled',
        params: 0,
        inputFormatter: []
      })
    ],
    properties: []
  }, {
    property: 'ethcore',
    methods: [
      new Method({
        name: 'getNetPeers',
        call: 'ethcore_netPeers',
        params: 0,
        outputFormatter: x => x
      }),
      new Method({
        name: 'getNetChain',
        call: 'ethcore_netChain',
        params: 0,
        outputFormatter: x => x
      }),
      new Method({
        name: 'gasPriceStatistics',
        call: 'ethcore_gasPriceStatistics',
        params: 0,
        outputFormatter: a => a.map(web3.toBigNumber)
      }),
      new Method({
        name: 'unsignedTransactionsCount',
        call: 'ethcore_unsignedTransactionsCount',
        params: 0,
        inputFormatter: []
      })
    ],
    properties: []
  }];
}
