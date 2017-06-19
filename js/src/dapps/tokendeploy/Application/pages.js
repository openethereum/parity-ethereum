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

const PAGES = [
  {
    path: 'overview',
    title: 'Overview',
    byline: 'Display all the current information relating to your own deployed tokens',
    description: 'View the total number of tokens in circulation, the number of different tokens associated with your accounts, as well as the types of tokens created by you. In addition, view the balances associated with your accounts in relation to the total in circulation.'
  },
  {
    path: 'transfer',
    title: 'Transfer',
    byline: 'Send tokens associated with your accounts to other addresses',
    description: 'Send any tokens created by you or received from others. In addition, have a bird\'s eye view of all events relating to token transfers, be it yours, created by others, either local or globally available on the network.'
  },
  {
    path: 'deploy',
    title: 'Deploy',
    byline: 'Deploy a new token to the network',
    description: 'Token registration has never been this easy. Select the name for your token, the TLA and the number of tokens in circulation. Start sending the tokens to contacts right from this interface. Optionally you can register the token with the Token Registry which would allow you to transact in tokens from anywhere these transactions are allowed.'
  }
];

export default PAGES;
