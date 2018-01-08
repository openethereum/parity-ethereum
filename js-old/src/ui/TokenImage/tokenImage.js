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
import { bindActionCreators } from 'redux';

import { fetchTokens } from '~/redux/providers/tokensActions';
import unknownImage from '~/../assets/images/contracts/unknown-64x64.png';

class TokenImage extends Component {
  static contextTypes = {
    api: PropTypes.object
  };

  static propTypes = {
    image: PropTypes.string,
    token: PropTypes.shape({
      image: PropTypes.string,
      address: PropTypes.string
    }).isRequired,
    fetchTokens: PropTypes.func.isRequired
  };

  state = {
    error: false
  };

  componentWillMount () {
    const { token } = this.props;

    if (token.native) {
      return;
    }

    if (!token.fetched) {
      if (!Number.isFinite(token.index)) {
        return console.warn('no token index', token);
      }

      this.props.fetchTokens([ token.index ]);
    }
  }

  render () {
    const { error } = this.state;
    const { image, token } = this.props;

    const imageurl = token.image || image;
    let imagesrc = unknownImage;

    if (imageurl && !error) {
      imagesrc = imageurl;
    }

    return (
      <img
        alt={ token.name }
        onError={ this.handleError }
        src={ imagesrc }
      />
    );
  }

  handleError = () => {
    this.setState({ error: true });
  };
}

function mapStateToProps (iniState) {
  const { images } = iniState;

  return (_, props) => {
    const { token } = props;

    return { image: images[token.address] };
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    fetchTokens
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TokenImage);
