import { Provider } from 'react-redux';
import ReactDOM from 'react-dom';
import React from 'react';

import localStore from 'store';

import './index.html';
import './index.css';
import '!file-loader?name=icon.png!./icon.png';
import 'dapp-styles/dapp-styles.less';
import './env-specific';

import Web3 from 'web3'; // must b after ./test otherwise it breaks
import middlewares from './middleware';
import Routes from './routes';
import MuiThemeProvider from './components/MuiThemeProvider';

import configure from './store';
import { Web3Provider } from './provider/web3-provider';
import EthcoreWeb3 from './provider/web3-ethcore-provider';
import { initAppAction } from './actions/app';

const web3 = new Web3(new Web3.providers.HttpProvider(process.env.RPC_ADDRESS || '/rpc/'));

const store = configure(middlewares(web3));

ReactDOM.render(
  <Provider store={ store }>
    <MuiThemeProvider>
      <Routes store={ store } />
    </MuiThemeProvider>
  </Provider>,
  document.getElementById('root')
);

const ethcoreWeb3 = new EthcoreWeb3(web3);
new Web3Provider(web3, ethcoreWeb3, store).start();

(window || global).store = localStore;
(window || global).web3 = web3;

store.dispatch(initAppAction());
