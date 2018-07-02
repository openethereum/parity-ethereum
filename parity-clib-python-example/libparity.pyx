cdef extern from "include/parity.h":
    ctypedef struct ParityParams:
        void* configuration
    int parity_config_from_cli(char** args, size_t* arg_lens, size_t len, void** out)
    void parity_config_destroy(void* cfg)
    int parity_start(const ParityParams* params, void** out)
    void parity_destroy(void* parity)
    int parity_rpc(void* parity, const char* rpc, size_t len, char* out_str, size_t* out_len)

#cdef class ParityLibParams:
#    cdef void* confguration
#    def __cinit__(self, configuration, ):
#        self.configuration = 
from cpython.mem cimport PyMem_Malloc, PyMem_Realloc, PyMem_Free
from libc.stdlib cimport malloc, free

#@cython.no_gc
cdef class ParityInterface:
    cdef ParityParams* params
    cdef void* parity

    def __cinit__(self):
        self.params = <ParityParams*> PyMem_Malloc(sizeof(ParityParams))
        self.parity = PyMem_Malloc(sizeof(void*))

    def __init__(self, args):
        cdef size_t length 
        cdef char** c_args
        cdef size_t* arg_lens
        cdef void* out
        length = len(args)
        c_args = <char**>PyMem_Malloc(sizeof(char*)*length)
        arg_lens = <size_t*>PyMem_Malloc(sizeof(size_t)*length)
        for i in range(length):
            py_byte_string = args[i].encode('UTF-8')
            c_args[i] = <char*>py_byte_string
            arg_lens[i] = len(py_byte_string)
        params_success =  parity_config_from_cli(&(c_args[0]), &(arg_lens[0]), length, &out)
        PyMem_Free(c_args)
        PyMem_Free(arg_lens)
        self.params.configuration = out
        #paritylib_config_from_cli(self, 
        success = parity_start(self.params, &self.parity)
        print(success)

    def __dealloc__(self):
        PyMem_Free(self.params)
        PyMem_Free(self.parity)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        print("inside exit")

    def paritylib_destroy(self):
        parity_destroy(self.parity)

    def paritylib_rpc(self, rpc_call):
        cdef char* c_rpc_call = <char*>PyMem_Malloc(sizeof(char)*len(rpc_call))
        cdef size_t* rpc_call_len = <size_t*>PyMem_Malloc(sizeof(size_t*))
        cdef size_t* out_len = <size_t*>PyMem_Malloc(sizeof(size_t*))
        out_len[0] = 10240
        cdef char* out_str = <char*>PyMem_Malloc(out_len[0])
        cdef void* parity_instance = self.parity
        try:
            py_byte_string_in = rpc_call.encode('UTF-8')

            c_rpc_call = <char*>py_byte_string_in
            rpc_call_len[0] = <size_t>len(c_rpc_call)
            success = parity_rpc(&(parity_instance[0]), c_rpc_call, rpc_call_len[0], out_str, out_len)
            import timer
            timer.sleep(10)
        finally:
            PyMem_Free(out_str)
            PyMem_Free(out_len)
            PyMem_Free(rpc_call_len)
            PyMem_Free(c_rpc_call)
        #if success:
        #    try:
        #        py_byte_string = out_str[:out_len].decode('UTF-8')
        #    finally:
        #        pass
        #        #PyMem_Free(out_str)
        #        #PyMem_Free(c_rpc_call)
        return py_byte_string_in
    
        

