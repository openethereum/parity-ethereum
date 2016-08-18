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

    api.ethcore
      .registryAddress()
      .then((address) => {
        console.log('registry', address);

        this.registry = api.newContract(registry).at(address);
        return this.registry.named
          .getAddress
          .call({}, [Api.format.sha3('tokenreg'), 'A']);
      })
      .then((address) => {
        console.log('tokenreg', address);

        this.tokenreg = api.newContract(tokenreg).at(address);
        return this.tokenreg.named
          .tokenCount
          .call();
      })
      .then((tokenCount) => {
        console.log('tokenCount', tokenCount.toNumber());

        const promises = [];

        while (promises.length < tokenCount.toNumber()) {
          promises.push(this.tokenreg.named.token.call({}, [promises.length]));
        }

        return Promise.all(promises);
      })
      .then((tokens) => {
        console.log('tokens', tokens);

        const eip20s = [];
        const promises = [];

        tokens.forEach((token) => {
          console.log(token[0], token[1], token[2].toFormat(), token[3]);

          const contract = api.newContract(eip20).at(token[0]);

          eip20s.push(contract);
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
