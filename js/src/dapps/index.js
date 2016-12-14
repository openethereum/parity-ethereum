// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

module.exports = [
  { name: 'basiccoin', entry: 'basiccoin.js', title: 'Basic Token Deployment' },
  { name: 'dappreg', entry: 'dappreg.js', title: 'Dapp Registry' },
  { name: 'githubhint', entry: 'githubhint.js', title: 'GitHub Hint', secure: true },
  { name: 'localtx', entry: 'localtx.js', title: 'Local transactions Viewer', secure: true },
  { name: 'registry', entry: 'registry.js', title: 'Registry' },
  { name: 'signaturereg', entry: 'signaturereg.js', title: 'Method Signature Registry' },
  { name: 'tokenreg', entry: 'tokenreg.js', title: 'Token Registry' }
];
