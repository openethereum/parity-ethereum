# Based off example setup.py from pyo3
import sys

from setuptools import setup
from setuptools.command.test import test as TestCommand
from setuptools_rust import RustExtension

def get_py_version_cfgs():
    # For now each Cfg Py_3_X flag is interpreted as "at least 3.X"
    version = sys.version_info[0:2]

    if version[0] == 2:
        return ["--cfg=Py_2"]

    py3_min = 5
    out_cfg = []
    for minor in range(py3_min, version[1] + 1):
        out_cfg.append("--cfg=Py_3_%d" % minor)

    return out_cfg


install_requires = []
tests_require = install_requires + ["pytest", "pytest-benchmark"]

setup(
        name="parity-clib",
        version="2.5.0",
        packages=["parity"],
        rust_extensions=[
            RustExtension(
                "_parity", "../Cargo.toml", rustc_flags=get_py_version_cfgs()
            ),
        ],
        install_requires=install_requires,
        tests_require=tests_require,
        include_package_data=True,
        zip_safe=False,
)
