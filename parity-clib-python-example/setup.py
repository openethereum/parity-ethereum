from setuptools import setup, Extension, find_packages

from Cython.Build import cythonize
from Cython.Distutils import build_ext

ext = Extension('libparity',
                sources=['libparity.pyx'],
                libraries=['libparity.so',],
                library_dirs=['target/release',],
                include_dirs=['include',]
)

extensions = [ext,]

setup(
        name = "libparity",
        ext_modules = cythonize(extensions),
        cmdclass={'build_ext': build_ext},
)
