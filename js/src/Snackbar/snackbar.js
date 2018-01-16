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
import PropTypes from 'prop-types';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { closeSnackbar } from '@parity/shared/lib/redux/providers/snackbarActions';
import SnackbarUI from '@parity/ui/lib/Snackbar';

function Snackbar ({ closeSnackbar, cooldown = 3500, message, open = false }) {
  return (
    <SnackbarUI
      open={ open }
      message={ message }
      autoHideDuration={ cooldown }
      onRequestClose={ closeSnackbar }
    />
  );
}

Snackbar.propTypes = {
  closeSnackbar: PropTypes.func.isRequired,
  cooldown: PropTypes.number,
  message: PropTypes.any,
  open: PropTypes.bool
};

function mapStateToProps (state) {
  const { open, message, cooldown } = state.snackbar;

  return { open, message, cooldown };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    closeSnackbar
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Snackbar);
