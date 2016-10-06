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

const overlay = {
  position: 'fixed',
  top: 0,
  right: 0,
  bottom: 0,
  left: 0,
  background: 'rgba(255, 255, 255, 0.75)',
  zIndex: 20000
};

const modal = {
  position: 'fixed',
  top: 0,
  right: 0,
  bottom: 0,
  left: 0,
  zIndex: 20001
};

const body = {
  margin: '0 auto',
  padding: '2em 4em',
  textAlign: 'center',
  maxWidth: '40em',
  background: 'rgba(25, 25, 25, 0.75)',
  color: 'rgb(208, 208, 208)',
  boxShadow: 'rgba(0, 0, 0, 0.25) 0px 14px 45px, rgba(0, 0, 0, 0.22) 0px 10px 18px'
};

const header = {
  fontSize: '1.25em'
};

const info = {
  marginTop: '2em',
  lineHeight: '1.618em'
};

const icons = {
};

const icon = {
  display: 'inline-block',
  padding: '0.5em 1em 1em 1em',
  margin: '0 1em',
  background: 'rgba(200, 200, 200, 0.25)',
  borderRadius: '50%'
};

const iconName = {
};

const iconSvg = {
  width: '5.5em',
  height: '5.5em',
  margin: '0 0 -1em 0'
};

export {
  body, header, icons, icon, iconName, iconSvg, info, modal, overlay
};
