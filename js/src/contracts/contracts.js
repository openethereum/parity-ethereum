// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import DappReg from './dappreg';
import Registry from './registry';
import SignatureReg from './signaturereg';
import TokenReg from './tokenreg';
import GithubHint from './githubhint';
import * as verification from './verification';
import BadgeReg from './badgereg';

let instance = null;

export default class Contracts {
  constructor (api) {
    instance = this;

    this._api = api;
    this._registry = new Registry(api);
    this._dappreg = new DappReg(api, this._registry);
    this._signaturereg = new SignatureReg(api, this._registry);
    this._tokenreg = new TokenReg(api, this._registry);
    this._githubhint = new GithubHint(api, this._registry);
    this._badgeReg = new BadgeReg(api, this._registry);
  }

  get registry () {
    return this._registry;
  }

  get badgeReg () {
    return this._badgeReg;
  }

  get dappReg () {
    return this._dappreg;
  }

  get signatureReg () {
    return this._signaturereg;
  }

  get tokenReg () {
    return this._tokenreg;
  }

  get githubHint () {
    return this._githubhint;
  }

  get smsVerification () {
    return verification;
  }

  get emailVerification () {
    return verification;
  }

  static create (api) {
    if (instance) {
      return instance;
    }

    return new Contracts(api);
  }

  static get () {
    return instance;
  }
}
