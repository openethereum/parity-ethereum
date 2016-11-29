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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { hashToImageUrl } from '../../redux/providers/imagesReducer';
import { fetchCertifications } from '../../redux/providers/certifications/actions';

const defaultIcon = '/api/content/371b226f700d8577fe849d7b2729bc2e4be8c06c38159fb880a6a0cc276af012';

import styles from './certifications.css';

class Certifications extends Component {
  static propTypes = {
    account: PropTypes.string.isRequired,
    certifications: PropTypes.object.isRequired,
    dappsUrl: PropTypes.string.isRequired,

    fetchCertifications: PropTypes.func.isRequired
  }

  componentWillMount () {
    const { account, fetchCertifications } = this.props;
    fetchCertifications(account);
  }

  render () {
    const { account, certifications } = this.props;
    const certificationsOfAccount = certifications[account] || [];

    if (certificationsOfAccount.length === 0) {
      return null;
    }

    return (
      <div className={ styles.certifications }>
        { certificationsOfAccount.map(this.renderCertification) }
      </div>
    );
  }

  renderCertification = (certification) => {
    const { name, title, icon } = certification;
    const { dappsUrl } = this.props;

    const classNames = `${styles.certification} ${!icon ? styles.noIcon : ''}`;
    const img = dappsUrl + (icon ? hashToImageUrl(icon) : defaultIcon);
    return (
      <div className={ classNames } key={ name }>
        <img className={ styles.icon } src={ img } />
        <div className={ styles.text }>{ title || name }</div>
      </div>
    );
  }
}

function mapStateToProps (state) {
  return { certifications: state.certifications };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({ fetchCertifications }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Certifications);
