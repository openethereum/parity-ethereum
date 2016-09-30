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
    color: 'green',
    title: 'Overview',
    byline: 'Display all the current information relating to your own deployed tokens'
  },
  {
    path: 'send',
    color: 'red',
    title: 'Send',
    byline: 'Send tokens associated with your accounts to other addresses'
  },
  {
    path: 'events',
    color: 'blue',
    title: 'Events',
    byline: 'Track the events for your tokens, showing actions as they hapenned'
  },
  {
    path: 'deploy',
    color: 'purple',
    title: 'Deploy',
    byline: 'Deploy a new token to the network'
  },
  {
    path: 'status',
    color: 'orange',
    title: 'Status',
    byline: 'Show the status of all network tokens deployed with this application'
  }
];

export default PAGES;
