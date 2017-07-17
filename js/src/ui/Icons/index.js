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
import { Icon } from 'semantic-ui-react';

export function createIcon (name, props = {}) {
  return <Icon name={ name } { ...props } />;
}

export const AccountsIcon = (props) => createIcon('university', props);
export const AddIcon = (props) => createIcon('plus', props);
export const AddressIcon = (props) => createIcon('address book outline', props);
export const AppsIcon = (props) => createIcon('sitemap', props);
export const AttachFileIcon = (props) => createIcon('attach', props);
export const BackgroundIcon = (props) => createIcon('image', props);
export const CancelIcon = (props) => createIcon('cancel', props);
export const CheckIcon = (props) => createIcon('check', props);
export const CheckboxTickedIcon = (props) => createIcon('checkmark box', props);
export const CheckboxUntickedIcon = (props) => createIcon('square outline', props);
export const CloseIcon = (props) => createIcon('close', props);
export const CompareIcon = (props) => createIcon('exchange', props);
export const ComputerIcon = (props) => createIcon('desktop', props);
export const ContractIcon = (props) => createIcon('code', props);
export const CopyIcon = (props) => createIcon('copy', props);
export const DashboardIcon = (props) => createIcon('cubes', props);
export const DoneIcon = CheckIcon;
export const DeleteIcon = (props) => createIcon('trash', props);
export const DevelopIcon = (props) => createIcon('connectdevelop', props);
export const DialIcon = (props) => createIcon('text telephone', props);
export const EditIcon = (props) => createIcon('edit', props);
export const ErrorIcon = (props) => createIcon('exclamation circle', props);
export const EthernetIcon = (props) => createIcon('wifi', props);
export const FileIcon = (props) => createIcon('file outline', props);
export const FileDownloadIcon = (props) => createIcon('download', props);
export const FileUploadIcon = (props) => createIcon('upload', props);
export const FingerprintIcon = (props) => createIcon('target', props);
export const GasIcon = (props) => createIcon('settings', props);
export const GotoIcon = (props) => createIcon('arrow circle right', props);
export const InfoIcon = (props) => createIcon('info circle', props);
export const KeyIcon = (props) => createIcon('key', props);
export const KeyboardIcon = (props) => createIcon('keyboard', props);
export const LinkIcon = (props) => createIcon('linkify', props);
export const ListIcon = (props) => createIcon('list ul', props);
export const LockedIcon = (props) => createIcon('unlock alternate', props);
export const MembershipIcon = (props) => createIcon('id card outline', props);
export const MethodsIcon = (props) => createIcon('map signs', props);
export const MoveIcon = (props) => createIcon('move', props);
export const NextIcon = (props) => createIcon('chevron right', props);
export const PauseIcon = (props) => createIcon('pause', props);
export const PlayIcon = (props) => createIcon('play', props);
export const PrevIcon = (props) => createIcon('chevron left', props);
export const PrintIcon = (props) => createIcon('print', props);
export const QrIcon = (props) => createIcon('qrcode', props);
export const RefreshIcon = (props) => createIcon('refresh', props);
export const RemoveIcon = (props) => createIcon('remove', props);
export const ReorderIcon = (props) => createIcon('align justify', props);
export const ReplayIcon = (props) => createIcon('retweet', props);
export const SaveIcon = (props) => createIcon('save', props);
export const SearchIcon = (props) => createIcon('search', props);
export const SendIcon = (props) => createIcon('send', props);
export const SettingsIcon = (props) => createIcon('settings', props);
export const SnoozeIcon = (props) => createIcon('clock', props);
export const SortIcon = (props) => createIcon('filter', props);
export const StarIcon = (props) => createIcon('star', props);
export const StatusIcon = (props) => createIcon('signal', props);
export const UnlockedIcon = (props) => createIcon('unlock', props);
export const UpdateIcon = (props) => createIcon('cloud download', props);
export const UpdateWaitIcon = (props) => createIcon('wait', props);
export const VisibleIcon = (props) => createIcon('eye', props);
export const VerifyIcon = (props) => createIcon('shield', props);
export const VpnIcon = (props) => createIcon('world', props);
