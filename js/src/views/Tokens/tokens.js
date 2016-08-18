import React, { Component } from 'react';

import Api from '../../api';
import Container from '../../ui/Container';

import { eip20, registry, tokenreg } from '../../services/contracts';

export default class Tokens extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  componentDidMount () {
    const api = this.context.api;
    this.contracts = {};

    api.ethcore
      .registryAddress()
      .then((address) => {
        this.contracts.registry = api.newContract(registry).at(address);
        return this.contracts.registry.named
          .getAddress
          .call({}, [Api.format.sha3('tokenreg'), 'A']);
      })
      .then((address) => {
        this.contracts.tokenreg = api.newContract(tokenreg).at(address);
        return this.contracts.tokenreg.named
          .tokenCount
          .call();
      })
      .then((tokenCount) => {
        const promises = [];

        while (promises.length < tokenCount.toNumber()) {
          promises.push(this.contracts.tokenreg.named.token.call({}, [promises.length]));
        }

        return Promise.all(promises);
      })
      .then((tokens) => {
        this.eip20s = [];
        const promises = [];

        tokens.forEach((token) => {
          console.log(token[0], token[1], token[2].toFormat(), token[3]);

          const contract = api.newContract(eip20).at(token[0]);

          this.eip20s.push({
            token: token[1],
            type: token[3],
            image: `images/tokens/${token[3].toLowerCase()}-32x32.png`,
            contract
          });
          promises.push(contract.named.totalSupply.call());
        });

        return Promise.all(promises);
      })
      .then((supplies) => {
        console.log('supplies', supplies.map((supply) => supply.toFormat()));
      })
      .catch((error) => {
        console.error(error);
      });

    // this.unicorns = api.newContract(eip20).at(UNICORNS);
    // this.unicorns.named
    //   .totalSupply
    //   .call()
    //   .then((totalSupply) => console.log('totalSupply', totalSupply.toString()));
  }

  render () {
    return (
      <Container>
        <div>the token dapp interface should go in here, we need</div>
        <ul>
          <li>a basic contract</li>
          <li>deploy it and</li>
          <li>then go about playing and seeing what is the best way to pull everything together...</li>
        </ul>
      </Container>
    );
  }
}
