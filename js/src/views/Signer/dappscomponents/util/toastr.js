// pass this to action creator's meta (third argument) to make action toastable
export function withToastr (msgFunc, type = 'default') {
  return function (msg) {
    return {
      toastr: {
        msg: msgFunc(msg),
        type
      }
    };
  };
}
