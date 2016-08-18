import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import darkBaseTheme from 'material-ui/styles/baseThemes/darkBaseTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';

import Api from '../../api';
import { eip20Abi, registryAbi, tokenRegAbi } from '../../services/abi';
import { TooltipOverlay } from '../../ui/Tooltip';

import { FirstRun } from '../../modals';
import Status from './Status';
import TabBar from './TabBar';

import styles from './style.css';

const lightTheme = getMuiTheme(lightBaseTheme);
const muiTheme = getMuiTheme(darkBaseTheme);
const api = new Api(new Api.Transport.Http('/rpc/'));

muiTheme.stepper.textColor = '#eee';
muiTheme.stepper.disabledTextColor = '#777';
muiTheme.inkBar.backgroundColor = 'rgb(0, 151, 167)';
muiTheme.tabs = lightTheme.tabs;
muiTheme.tabs.backgroundColor = 'rgb(65, 65, 65)';
muiTheme.textField.disabledTextColor = muiTheme.textField.textColor;
muiTheme.toolbar = lightTheme.toolbar;
muiTheme.toolbar.backgroundColor = 'rgb(80, 80, 80)';

export default class Application extends Component {
  static childContextTypes = {
    api: PropTypes.object,
    tokens: PropTypes.array,
    muiTheme: PropTypes.object
  }

  static propTypes = {
    children: PropTypes.node
  }

  state = {
    showFirst: false,
    accounts: [],
    tokens: []
  }

  componentWillMount () {
    this.retrieveInfo();
  }

  render () {
    return (
      <TooltipOverlay>
        <div className={ styles.container }>
          <FirstRun
            onClose={ this.onCloseFirst }
            visible={ this.state.showFirst } />
          <TabBar />
          { this.props.children }
          <Status />
        </div>
      </TooltipOverlay>
    );
  }

  getChildContext () {
    return {
      api,
      tokens: this.state.tokens,
      muiTheme
    };
  }

  retrieveInfo () {
    const contracts = {};
    const tokens = [];

    Promise
      .all([
        api.personal.listAccounts(),
        api.ethcore.registryAddress()
      ])
      .then(([accounts, registryAddress]) => {
        this.setState({
          accounts,
          showFirst: accounts.length === 0
        });

        contracts.registry = api.newContract(registryAbi).at(registryAddress);

        return contracts.registry
          .getAddress.call({}, [Api.format.sha3('tokenreg'), 'A']);
      })
      .then((tokenregAddress) => {
        contracts.tokenreg = api.newContract(tokenRegAbi).at(tokenregAddress);

        return contracts.tokenreg
          .tokenCount.call();
      })
      .then((tokenCount) => {
        const promises = [];

        while (promises.length < tokenCount.toNumber()) {
          promises.push(contracts.tokenreg.token.call({}, [promises.length]));
        }

        return Promise.all(promises);
      })
      .then((_tokens) => {
        return Promise
          .all(_tokens.map((token) => {
            console.log(token[0], token[1], token[2].toFormat(), token[3]);

            const contract = api.newContract(eip20Abi).at(token[0]);

            tokens.push({
              address: token[0],
              format: token[2].toString(),
              image: `images/tokens/${token[3].toLowerCase()}-32x32.png`,
              supply: '0',
              token: token[1],
              type: token[3],
              contract
            });

            return contract.totalSupply.call();
          }));
      })
      .then((supplies) => {
        console.log('supplies', supplies.map((supply) => supply.toFormat()));

        supplies.forEach((supply, idx) => {
          tokens[idx].supply = supply.toString();
        });

        this.setState({
          tokens,
          contracts
        });
      })
      .catch((error) => {
        console.error(error);
      });
  }

  onCloseFirst = () => {
    this.setState({
      showFirst: false
    });
  }
}
