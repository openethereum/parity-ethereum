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

import React, { Component, PropTypes } from 'react';

import { Container } from '../../../ui';

import layout from '../layout.css';

export default class Views extends Component {
  static propTypes = {
  }

  state = {
  }

  render () {
    return (
      <Container>
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <p>Manage the available application views, using only the parts of the application that is applicable to you.</p>
            <p>Are you an end-user? The defaults are setups for both beginner and advanced users alike.</p>
            <p>Are you a developer? Add some features to manage contracts are interact with application develoyments.</p>
            <p>Are you a miner or run a large-scale node? Add the features to give you all the information needed to watch the node operation.</p>
          </div>
          <div className={ layout.details }>
            details goes here
          </div>
        </div>
      </Container>
    );
  }
}
