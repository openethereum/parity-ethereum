import React from 'react';
import { Provider } from 'react-redux';
import ReactDOM from 'react-dom';

import BigNumber from 'bignumber.js';

// Needed for onTouchTap
// http://stackoverflow.com/a/34015469/988941
import injectTapEventPlugin from 'react-tap-event-plugin';

import './chromeExtension.css';
import MuiThemeProvider from '../MuiThemeProvider';
import Web3Provider from '../Web3Provider';
import Web3 from 'web3';
import web3extensions from '../util/web3.extensions';

import Routes from './routes';
import middlewares from './middlewares';
import createStore from './store';

const http = new Web3.providers.HttpProvider(process.env.RPC_ADDRESS || '/rpc/');
const web3 = new Web3(http);
web3extensions(web3)
  .map(extension => web3._extend(extension));

const store = createStore(middlewares());

injectTapEventPlugin();

ReactDOM.render(
  <Provider store={ store }>
    <Web3Provider web3={ web3 }>
      <MuiThemeProvider>
        <Routes store={ store } />
      </MuiThemeProvider>
    </Web3Provider>
  </Provider>,
  document.querySelector('#root')
);

global.web3 = web3;
global.BigNumber = BigNumber;
