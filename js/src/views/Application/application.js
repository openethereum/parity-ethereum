import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import Api from '../../api';
import { eip20Abi, registryAbi, tokenRegAbi } from '../../util/abi';

import Container from './Container';
import DappContainer from './DappContainer';
import FrameError from './FrameError';
import Status, { updateNodeStatus } from './Status';
import TabBar from './TabBar';

import imagesEthereum32 from '../../images/contracts/ethereum-32.png';
import imagesEthereum56 from '../../images/contracts/ethereum-56.png';
import imagesGavcoin32 from '../../images/contracts/gavcoin-32.png';
import imagesGavcoin56 from '../../images/contracts/gavcoin-56.png';

const api = new Api(new Api.Transport.Http('/rpc/'));
const inFrame = window.parent !== window && window.parent.frames.length !== 0;

// TODO: Images should not be imported like this, should be via the content from GitHubHint contract (here until it is ready)
const images = {
  ethereum: {
    small: imagesEthereum32,
    normal: imagesEthereum56
  },
  gavcoin: {
    small: imagesGavcoin32,
    normal: imagesGavcoin56
  }
};
const ETH_TOKEN = {
  images: images.ethereum,
  name: 'Ethereum',
  tag: 'ΞTH'
};

let lastBlockNumber = new BigNumber(-1);

class Application extends Component {
  static childContextTypes = {
    api: PropTypes.object,
    accounts: PropTypes.array,
    balances: PropTypes.object,
    contacts: PropTypes.array,
    tokens: PropTypes.array
  }

  static propTypes = {
    children: PropTypes.node,
    netChain: PropTypes.string,
    isTest: PropTypes.bool,
    onUpdateNodeStatus: PropTypes.func,
    pending: PropTypes.array
  }

  state = {
    showFirstRun: false,
    accounts: [],
    balances: {},
    contacts: [],
    contracts: [],
    tokens: []
  }

  componentWillMount () {
    this.retrieveAccounts();
    this.retrieveTokens();
    this.pollStatus();
  }

  render () {
    const { children, pending, netChain, isTest } = this.props;
    const { showFirstRun } = this.state;
    const [root] = (window.location.hash || '').replace('#/', '').split('/');

    if (inFrame) {
      return (
        <FrameError />
      );
    } else if (root === 'app') {
      return (
        <DappContainer>
          { children }
        </DappContainer>
      );
    }

    return (
      <Container
        showFirstRun={ showFirstRun }
        onCloseFirstRun={ this.onCloseFirstRun }>
        <TabBar
          netChain={ netChain }
          isTest={ isTest }
          pending={ pending } />
        { children }
        <Status />
      </Container>
    );
  }

  getChildContext () {
    const { accounts, balances, contacts, tokens } = this.state;

    return {
      api,
      accounts,
      balances,
      contacts,
      tokens
    };
  }

  retrieveAccounts = () => {
    let { showFirstRun } = this.state;
    const nextTimeout = () => setTimeout(this.retrieveAccounts, 1000);

    Promise
      .all([
        api.personal.listAccounts(),
        api.personal.accountsInfo()
      ])
      .then(([addresses, infos]) => {
        const contacts = [];
        const accounts = [];

        Object.keys(infos).forEach((address) => {
          const { name, meta, uuid } = infos[address];

          if (uuid) {
            const account = this.state.accounts.find((_account) => _account.uuid === uuid) || {
              address,
              uuid
            };

            accounts.push(Object.assign(account, {
              meta,
              name
            }));
          } else {
            const contact = this.state.contacts.find((_contact) => _contact.address === address) || {
              address
            };

            contacts.push(Object.assign(contact, {
              meta,
              name
            }));
          }
        });

        if (!accounts.length) {
          showFirstRun = true;
        }

        this.setState({
          accounts,
          contacts,
          showFirstRun
        }, nextTimeout);
      })
      .catch((error) => {
        console.error('retrieveAccounts', error);

        nextTimeout();
      });
  }

  retrieveBalances = () => {
    const { accounts, contacts, tokens } = this.state;
    const balances = {};
    const addresses = accounts.concat(contacts).map((account) => account.address);

    return Promise
      .all(
        addresses.map((address) => {
          return Promise.all([
            api.eth.getBalance(address),
            api.eth.getTransactionCount(address)
          ]);
        })
      )
      .then((balancesTxCounts) => {
        return Promise.all(
          balancesTxCounts.map(([balance, txCount], idx) => {
            const address = addresses[idx];
            const { isTest } = this.props;

            balances[address] = {
              txCount: txCount.sub(isTest ? 0x100000 : 0),
              tokens: [{
                token: ETH_TOKEN,
                value: balance.toString()
              }]
            };

            return Promise.all(
              tokens.map((token) => {
                return token.contract.instance.balanceOf.call({}, [address]);
              })
            );
          })
        );
      })
      .then((tokenBalances) => {
        addresses.forEach((address, idx) => {
          const balanceOf = tokenBalances[idx];
          const balance = balances[address];

          tokens.forEach((token, tidx) => {
            balance.tokens.push({
              token,
              value: balanceOf[tidx].toString()
            });
          });
        });

        this.setState({
          balances
        });
      })
      .catch((error) => {
        console.error('retrieveBalances', error);
      });
  }

  retrieveTokens = () => {
    api.ethcore
      .registryAddress()
      .then((registryAddress) => {
        const registry = api.newContract(registryAbi, registryAddress);

        return registry.instance.getAddress.call({}, [api.format.sha3('tokenreg'), 'A']);
      })
      .then((tokenregAddress) => {
        const tokenreg = api.newContract(tokenRegAbi, tokenregAddress);

        return tokenreg.instance.tokenCount
          .call()
          .then((numTokens) => {
            const promises = [];

            while (promises.length < numTokens.toNumber()) {
              promises.push(tokenreg.instance.token.call({}, [promises.length]));
            }

            return Promise.all(promises);
          });
      })
      .then((tokens) => {
        this.setState({
          tokens: tokens.map((token) => {
            const [address, tag, format, name] = token;

            return {
              address,
              name,
              tag,
              format: format.toString(),
              images: images[name.toLowerCase()],
              contract: api.newContract(eip20Abi, address)
            };
          })
        }, this.retrieveBalances);
      })
      .catch((error) => {
        console.error('retrieveTokens', error);
      });
  }

  pollStatus () {
    const { onUpdateNodeStatus } = this.props;
    const nextTimeout = () => setTimeout(() => this.pollStatus(), 1000);

    Promise
      .all([
        api.eth.blockNumber(),
        api.web3.clientVersion(),
        api.ethcore.netChain(),
        api.ethcore.netPeers(),
        api.eth.syncing()
      ])
      .then(([blockNumber, clientVersion, netChain, netPeers, syncing]) => {
        const isTest = netChain === 'morden' || netChain === 'testnet';

        onUpdateNodeStatus({
          blockNumber,
          clientVersion,
          netChain,
          netPeers,
          isTest,
          syncing
        });

        if (blockNumber.gt(lastBlockNumber)) {
          lastBlockNumber = blockNumber;
          this.retrieveBalances();
        }

        nextTimeout();
      })
      .catch((error) => {
        console.error('pollStatus', error);

        nextTimeout();
      });
  }

  onCloseFirstRun = () => {
    this.setState({
      showFirstRun: false
    });
  }
}

function mapStateToProps (state) {
  const { netChain, isTest } = state.nodeStatus;
  const { pending } = state.signerRequests;

  return {
    netChain,
    isTest,
    pending
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    onUpdateNodeStatus: updateNodeStatus
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Application);
