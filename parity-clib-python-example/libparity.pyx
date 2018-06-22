from libc.stdlib cimport free
cdef extern from "parity.h":
    ctypedef struct ParityParams:
        pass
    int parity_config_from_cli(char** args, size_t* arg_lens, size_t len, void** out)
    void parity_config_destroy(void* cfg)
    int parity_start(const ParityParams* params, void** out)
    void parity_destroy(void* parity)
    int parity_rpc(void* parity, char* rpc, size_t len, char* out_str, size_t* out_len)

#cdef class ParityLibParams:
#    cdef void* confguration
#    def __cinit__(self, configuration, ):
#        self.configuration = 
from cpython.mem cimport PyMem_Malloc, PyMem_Realloc, PyMem_Free
cdef class ParityInterface:
    cdef ParityParams* params
    cdef void* parity

    def __cinit__(self):
        self.params = <ParityParams*> PyMem_Malloc(4*sizeof(double))

    def __init__(self, args):
        cdef size_t length 
        cdef size_t* arg_lens  
        cdef char** c_args 
        cdef void* out
        length = args.len()
        for i in range(length):
            py_byte_string = args[i].encode('UTF-8')
            c_args[i] = py_byte_string
            arg_lens[i] = py_byte_string.len()
        params_success =  parity_config_from_cli(&(c_args[0]), &(arg_lens[0]), length, &out)
        self.params = <ParityParams*>out
        #paritylib_config_from_cli(self, 
        success = parity_start(self.params, &self.parity)
        print(success)

    def paritylib_destroy(self):
        return <object>parity_destroy(self.parity)

    def paritylib_rpc(self, unicode rpc_call):
        cdef char* c_rpc_call 
        cdef size_t rpc_call_len = rpc_call.len()
        cdef size_t* out_len
        cdef char* out_str  
        c_rpc_call[0] = rpc_call.encode('UTF-8')
        success = parity_rpc(&self.parity, c_rpc_call, rpc_call_len, out_str, out_len)
        if success:
            out_str_len = out_len[0]
            try:
                py_byte_string = out_str[:out_str_len]
            finally:
                free(out_str)
            return py_byte_string    
    
        

