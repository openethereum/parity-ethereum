import { Component } from 'react';

const isProd = process.env.NODE_ENV === 'production';

// Component utils for integration tests hooks.
const TEST_HOOK = 'data-test';
Component.prototype._test = isProd ? noop : testHook;
Component.prototype._testInherit = isProd ? noop : testHookInherit;

function noop (name) {}

function testHookInherit (name) {
  let hook = this.props[TEST_HOOK];
  if (name) {
    hook += `-${name}`;
  }
  return {
    [TEST_HOOK]: hook
  };
}

function testHook (name) {
  let hook = this.constructor.name;
  if (name) {
    hook += `-${name}`;
  }
  return {
    [TEST_HOOK]: hook
  };
}
