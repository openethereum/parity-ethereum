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
import Chip from 'material-ui/Chip';

import { Container, ContainerTitle } from '../../../ui';

import styles from '../contract.css';

export default class Queries extends Component {
  static propTypes = {
    contract: PropTypes.object,
    values: PropTypes.object
  }

  render () {
    const { contract } = this.props;

    if (!contract) {
      return null;
    }

    const queries = contract.functions
      .filter((fn) => fn.constant)
      .sort(this._sortEntries)
      .map((fn) => this.renderQuery(fn));

    return (
      <Container>
        <ContainerTitle title='queries' />
        <div className={ styles.methods }>
          { queries }
        </div>
      </Container>
    );
  }

  renderQuery (fn) {
    const { values } = this.props;

    const value = values[fn.name]
      ? (<Chip>{ values[fn.name].toString() }</Chip>)
      : null;

    return (
      <div
        key={ fn.signature }
        className={ styles.method }>
        <p>{ fn.name }</p>
        { value }
      </div>
    );
  }

  _sortEntries (a, b) {
    return a.name.localeCompare(b.name);
  }
}
