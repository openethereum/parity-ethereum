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

import { stringify } from 'querystring';
import React from 'react';

import styles from './styles.css';

export const howItWorks = (
  <div>
    <p>The following steps will let you prove that you control both an account and an e-mail address.</p>
    <ol className={ styles.list }>
      <li>You send a verification request to a specific contract.</li>
      <li>Our server puts a puzzle into this contract.</li>
      <li>The code you receive via e-mail is the solution to this puzzle.</li>
    </ol>
  </div>
);

export const termsOfService = (
  <ul>
    <li>todo</li>
  </ul>
);

export const postToServer = (query, isTestnet = false) => {
  const port = isTestnet ? 28443 : 18443;
  query = stringify(query);
  return fetch(`https://email-verification.parity.io:${port}/?` + query, {
    method: 'POST', mode: 'cors', cache: 'no-store'
  })
  .then((res) => {
    return res.json().then((data) => {
      if (res.ok) {
        return data.message;
      }
      throw new Error(data.message || 'unknown error');
    });
  });
};
