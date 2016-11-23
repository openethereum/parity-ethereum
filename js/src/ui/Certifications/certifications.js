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

import { hashToImageUrl } from '../../redux/providers/imagesReducer';

import styles from './certifications.css';

export default class Certifications extends Component {
  static propTypes = {
    certifications: PropTypes.array.isRequired,
    dappsUrl: PropTypes.string.isRequired
  }

  render () {
    const { certifications } = this.props;

    if (certifications.length === 0) {
      return null;
    }

    return (
      <div className={ styles.certifications }>
        { certifications.map(this.renderCertification) }
      </div>
    );
  }

  renderCertification = (certification) => {
    const { name, icon } = certification;
    const { dappsUrl } = this.props;

    return (
      <div className={ styles.certification } key={ name }>
        <img className={ styles.icon } src={ dappsUrl + hashToImageUrl(icon) } />
        { name }
      </div>
    );
  }
}
