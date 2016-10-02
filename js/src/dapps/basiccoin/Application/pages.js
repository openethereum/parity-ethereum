// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
    color: '#080',
    title: 'Overview',
    byline: 'Display all the current information relating to your own deployed tokens',
    description: 'View the total number of tokens in circulation, the number of different tokens associated with your accounts as well as the types of tokens created by you.'
  },
  {
    path: 'send',
    color: '#80f',
    title: 'Send',
    byline: 'Send tokens associated with your accounts to other addresses'
  },
  {
    path: 'events',
    color: '#808',
    title: 'Events',
    byline: 'Track the events for your tokens, showing actions as they hapenned'
  },
  {
    path: 'deploy',
    color: '#088',
    title: 'Deploy',
    byline: 'Deploy a new token to the network',
    description: 'Token registration has never been this easy. Select the name for your token, the TLA and the number of tokens in circulation. Start sending the tokens to contacts right from this interface. Optionally you can register the token with the Token Registry which would allow you to transaction in tokens from anywhere these transactions are allowed.'
  },
  {
    path: 'deployments',
    color: '#f80',
    title: 'Deployments',
    byline: 'Show the status of all network tokens deployed with this application',
    description: 'Showing all the token creation events, both for your tokens and tokens created by others on the network. This includes when, by whom, name & TLA as well as the global and local network status.'
  }
];

export default PAGES;
