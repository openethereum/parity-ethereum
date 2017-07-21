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

import { action, observable } from 'mobx';

export default class Store {
  @observable isDeleteVisible = false;
  @observable isEditVisible = false;
  @observable isExportVisible = false;
  @observable isFaucetVisible = false;
  @observable isFundVisible = false;
  @observable isPasswordVisible = false;
  @observable isTransferVisible = false;
  @observable isVerificationVisible = false;

  @action toggleDeleteDialog = () => {
    this.isDeleteVisible = !this.isDeleteVisible;
  }

  @action toggleEditDialog = () => {
    this.isEditVisible = !this.isEditVisible;
  }

  @action toggleExportDialog = () => {
    this.isExportVisible = !this.isExportVisible;
  }

  @action toggleFaucetDialog = () => {
    this.isFaucetVisible = !this.isFaucetVisible;
  }

  @action toggleFundDialog = () => {
    this.isFundVisible = !this.isFundVisible;
  }

  @action togglePasswordDialog = () => {
    this.isPasswordVisible = !this.isPasswordVisible;
  }

  @action toggleTransferDialog = () => {
    this.isTransferVisible = !this.isTransferVisible;
  }

  @action toggleVerificationDialog = () => {
    this.isVerificationVisible = !this.isVerificationVisible;
  }
}
