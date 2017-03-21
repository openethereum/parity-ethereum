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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';

import { hashToImageUrl } from '~/redux/providers/imagesReducer';

import defaultIcon from '../../../assets/images/certifications/unknown.svg';

import styles from './certifications.css';

class Certifications extends Component {
  static propTypes = {
    address: PropTypes.string.isRequired,
    certifications: PropTypes.array.isRequired,
    className: PropTypes.string,
    dappsUrl: PropTypes.string.isRequired,
    showOnlyIcon: PropTypes.bool
  }

  render () {
    const { certifications, className } = this.props;

    if (certifications.length === 0) {
      return null;
    }

    return (
      <div className={ [styles.certifications, className].join(' ') }>
        { certifications.map(this.renderCertification) }
      </div>
    );
  }

  renderCertification = (certification) => {
    const { name, icon } = certification;
    const { dappsUrl, showOnlyIcon } = this.props;

    const classNames = [
      showOnlyIcon
        ? styles.certificationIcon
        : styles.certification,
      !icon
        ? styles.noIcon
        : ''
    ];

    return (
      <div
        className={ classNames.join(' ') }
        key={ name }
      >
        <img
          className={ styles.icon }
          src={
            icon
              ? `${dappsUrl}${hashToImageUrl(icon)}`
              : defaultIcon
          }
        />
        { this.renderCertificationName(certification) }
      </div>
    );
  }

  renderCertificationName = (certification) => {
    const { showOnlyIcon } = this.props;
    const { name, title } = certification;

    if (showOnlyIcon) {
      return null;
    }

    return (
      <div className={ styles.text }>
        { title || name }
      </div>
    );
  }
}

function mapStateToProps (_, initProps) {
  const { address } = initProps;

  return (state) => {
    const certifications = state.certifications[address] || [];
    const dappsUrl = state.api.dappsUrl;

    return { certifications, dappsUrl };
  };
}

export default connect(
  mapStateToProps,
  null
)(Certifications);
