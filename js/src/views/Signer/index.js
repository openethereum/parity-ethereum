import './index.html';

const app = window.paritySigner; // exposed by app.js

initApp();

function initApp () {
  const initToken = window.localStorage.getItem('sysuiToken');
  // TODO [ToDr] Hardcoded address should replaced with options
  const address = process.env.NODE_ENV === 'production' ? window.location.host : '127.0.0.1:8180';
  app(initToken, tokenSetter, address);
}

function tokenSetter (token, cb) {
  window.localStorage.setItem('sysuiToken', token);
}
