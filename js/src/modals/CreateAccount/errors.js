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

import React from 'react';
import { FormattedMessage } from 'react-intl';

export default {
  invalidPassword:
    <FormattedMessage
      id='createAccount.error.invalidPassword'
      defaultMessage='you need to specify a password' />,

  noFile:
    <FormattedMessage
      id='createAccount.error.noFile'
      defaultMessage='select a valid wallet file to import' />,

  noKey:
    <FormattedMessage
      id='createAccount.error.noKey'
      defaultMessage='you need to provide the raw private key' />,

  noMatchPassword:
    <FormattedMessage
      id='createAccount.error.noMatchPassword'
      defaultMessage='the supplied passwords does not match' />,

  noName:
    <FormattedMessage
      id='createAccount.error.noName'
      defaultMessage='you need to specify a valid name for the account' />,

  noPassword:
    <FormattedMessage
      id='createAccount.error.noPassword'
      defaultMessage='supply a valid password to confirm the transaction' />,

  noPhrase:
    <FormattedMessage
      id='createAccount.error.noPhrase'
      defaultMessage='you need to specify the recovery phrase' />,

  invalidKey:
    <FormattedMessage
      id='createAccount.error.invalidKey'
      defaultMessage='the raw key needs to be hex, 64 characters in length and contain the prefix "0x"' />
};
