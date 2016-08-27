import 'isomorphic-fetch';

import es6Promise from 'es6-promise';
es6Promise.polyfill();

import Api from './api';

import IdentityIcon from './ui/IdentityIcon';

window.parity = {
  Api: Api,
  react: {
    IdentityIcon
  }
};
