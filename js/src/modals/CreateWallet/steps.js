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

const TITLE_DEPLOYMENT = (
  <FormattedMessage
    id='createWallet.steps.deployment'
    defaultMessage='wallet deployment'
  />
);
const TITLE_DETAILS = (
  (
    <FormattedMessage
      id='createWallet.steps.details'
      defaultMessage='wallet details'
    />
  )
);
const TITLE_INFO = (
  <FormattedMessage
    id='createWallet.steps.info'
    defaultMessage='wallet informaton'
  />
);
const TITLE_SCAN = (
  <FormattedMessage
    id='createWallet.steps.scan'
    defaultMessage='scan hardware'
  />
);
const TITLE_TYPE = (
  <FormattedMessage
    id='createWallet.steps.type'
    defaultMessage='wallet type'
  />
);

const STEPS_HARDWARE = {
  TYPE: {
    title: TITLE_TYPE
  },
  SCAN: {
    title: TITLE_SCAN,
    waiting: true
  }
};

const STEPS_MULTISIG = {
  TYPE: {
    title: TITLE_TYPE
  },
  DETAILS: {
    title: TITLE_DETAILS
  },
  DEPLOYMENT: {
    title: TITLE_DEPLOYMENT,
    waiting: true
  },
  INFO: {
    title: TITLE_INFO
  }
};

const STEPS_WATCH = {
  TYPE: {
    title: TITLE_TYPE
  },
  DETAILS: {
    title: TITLE_DETAILS
  },
  INFO: {
    gptitle: TITLE_INFO
  }
};

export {
  STEPS_HARDWARE,
  STEPS_MULTISIG,
  STEPS_WATCH
};
