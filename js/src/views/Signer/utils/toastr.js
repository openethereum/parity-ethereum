// pass this to action creator's meta (third argument) to make action toastable
export const withToastr = (msgFunc, type = 'default') => {
  return msg => {
    return {
      toastr: {
        msg: msgFunc(msg),
        type
      }
    };
  };
};
