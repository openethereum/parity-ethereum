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

import React from 'react';
import { FormattedMessage } from 'react-intl';

export default {
  duplicateName: (
    <FormattedMessage
      id='errors.duplicateName'
      defaultMessage='the name already exists'
    />
  ),

  noFile: (
    <FormattedMessage
      id='errors.noFile'
      defaultMessage='select a valid wallet file to import'
    />
  ),

  noKey: (
    <FormattedMessage
      id='errors.noKey'
      defaultMessage='you need to provide the raw private key'
    />
  ),

  noMatchPassword: (
    <FormattedMessage
      id='errors.noMatchPassword'
      defaultMessage='the supplied passwords does not match'
    />
  ),

  noMatchBackupPhrase: (
    <FormattedMessage
      id='errors.noMatchBackupPhrase'
      defaultMessage='the supplied recovery phrase does not match'
    />
  ),

  noMatchPhraseBackedUp: (
    <FormattedMessage
      id='errors.noMatchPhraseBackedUp'
      defaultMessage='type "I have written down the phrase"'
    />
  ),

  noName: (
    <FormattedMessage
      id='errors.noName'
      defaultMessage='you need to specify a valid name'
    />
  ),

  invalidKey: (
    <FormattedMessage
      id='errors.invalidKey'
      defaultMessage='the raw key needs to be hex, 64 characters in length and contain the prefix "0x"'
    />
  )

};
