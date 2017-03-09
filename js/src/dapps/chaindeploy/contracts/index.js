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

import * as badgereg from './badgereg';
import * as dappreg from './dappreg';
import * as gavcoin from './gavcoin';
import * as githubhint from './githubhint';
import * as jgvoting from './jg-voting';
import * as registry from './registry';
import * as signaturereg from './signaturereg';
import * as tokendeployMgr from './tokendeployMgr';
import * as tokendeployReg from './tokendeployReg';
import * as tokenreg from './tokenreg';
import * as verifyEmail from './verifyEmail';
import * as verifySms from './verifySms';
import * as wallet from './wallet';

const contracts = [
  // builtin
  githubhint,
  badgereg,
  dappreg,
  signaturereg,
  tokenreg,
  tokendeployReg,
  tokendeployMgr,
  verifyEmail,
  verifySms,
  wallet,

  // external
  gavcoin,
  jgvoting
];

export {
  contracts,
  registry
};
