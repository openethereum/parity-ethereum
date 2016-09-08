import { withToastr } from './toastr';

describe('UTIL TOASTR', () => {
  it('should return toastr object with type set to default, when no type is passed', () => {
    // given
    const msg = 'test';
    const msgFunc = x => `some text and ${x}`;

    // when
    const res = withToastr(msgFunc)(msg);

    // then
    expect(res).to.eql({
      toastr: {
        msg: `some text and ${msg}`,
        type: 'default'
      }
    });
  });

  it('should return toastr object with type set to success, when success type is passed', () => {
    // given
    const msg = 'test';
    const type = 'success';
    const msgFunc = x => `some text and ${x}`;

    // when
    const res = withToastr(msgFunc, type)(msg);

    // then
    expect(res).to.eql({
      toastr: {
        msg: `some text and ${msg}`,
        type
      }
    });
  });
});
