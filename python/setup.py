import os
import subprocess
import sys
from distutils.extension import Extension

try:
    from wheel.bdist_wheel import bdist_wheel
except ImportError:
    bdist_wheel = None

from setuptools import setup
from distutils.command.build_py import build_py
from distutils.command.build_ext import build_ext

# Build with clang if not otherwise specified.
if os.environ.get('OPENDP_MANYLINUX') == '1':
    os.environ.setdefault('CC', 'gcc')
    os.environ.setdefault('CXX', 'g++')
else:
    os.environ.setdefault('CC', 'clang')
    os.environ.setdefault('CXX', 'clang++')

PACKAGE = 'libopendp'
OPENDP_DEBUG = os.environ.get("OPENDP_DEBUG") == "1"
OPENDP_BUILD = 'static' if os.environ.get('OPENDP_BUILD_STATIC') == '1' else 'dynamic'


def _get_rust_dir():
    """path to rust directory"""
    # default rust directory is in the sibling folder
    default_rust_dir = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "rust"))
    return os.environ.get("OPENDP_RUST_DIR", default_rust_dir)


def _get_rust_output_path():
    """path to the output of `cargo build`"""
    return os.path.join(_get_rust_dir(), "target", "debug" if OPENDP_DEBUG else "release", _get_lib_name())


platform_to_name = {
    'static': {
        "darwin": "libopendp_ffi.a",
        "linux": "libopendp_ffi.a",
        "win32": "opendp_ffi.a",
    },
    'dynamic': {
        "darwin": "libopendp_ffi.dylib",
        "linux": "libopendp_ffi.so",
        "win32": "opendp_ffi.dll",
    }
}


def _get_lib_name():
    if sys.platform not in platform_to_name[OPENDP_BUILD]:
        raise Exception("Platform not supported", sys.platform)
    return platform_to_name[OPENDP_BUILD][sys.platform]


def build_opendp(base_path):
    cmdline = ['cargo', 'build']
    if not OPENDP_DEBUG:
        cmdline.append('--release')
    if not sys.stdout.isatty():
        cmdline.append('--color=always')
    # rv = subprocess.Popen(cmdline, cwd=_get_rust_dir()).wait()
    # if rv != 0:
    #     sys.exit(rv)
    # CAUTION: this needs to happen after building rust, because openssl won't build with it
    os.environ.setdefault('CFLAGS', '-std=c++11')

    # src_path = _get_rust_output_path()
    # tgt_path = os.path.join(base_path, _get_lib_name())
    # if os.path.isfile(tgt_path):
    #     os.remove(tgt_path)
    # shutil.copy2(src_path, base_path)


class CustomBuildPy(build_py):
    def run(self):
        # https://stackoverflow.com/a/48942866/10221612
        # CAUTION: file movements in the build_py step need to happen after the extension module is built
        #     The extension module generates python code
        self.run_command("build_ext")
        return build_py.run(self)


class CustomBuildExt(build_ext):
    def run(self):
        build_opendp(self.build_temp)
        return build_ext.run(self)


cmdclass = {
    'build_ext': CustomBuildExt,
    'build_py': CustomBuildPy,
}

# CAUTION: opendp needs to be root-level because hiding it behind /src makes package_dir={'': 'src'}, which breaks -e
package_dir = {'': '.'}
# CAUTION: the .h file is generated in build.rs, from the build_ext step. The .h file is referenced by the .i swig file
header_dir = os.path.join(_get_rust_dir(), 'opendp-ffi', "include")
swig_interface_path = os.path.join(header_dir, 'libopendp.i')
# CAUTION: this gives clang/gcc enough information to find the location of opendp_ffi
os.environ['LIBRARY_PATH'] = os.path.dirname(_get_rust_output_path())

if os.environ.get('LIBOPENDP_MANYLINUX') == '1':
    # BUILD_STATIC = True
    opendp = Extension(
        name='_libopendp',
        # TODO
        # CentOS5 only has swig 1.3, which doesn't really work
        # Need to run `make python` in pasta-bindings before manylinux build
        sources=['/pasta-bindings/python/pasta_wrap.cpp'],
        language="c++",
        include_dirs=[header_dir],
        extra_compile_args=["-fPIC", "-c", "-g"],
        extra_link_args=["-shared"],
        # extra_objects=['libopendp.a'], # This is handled by CustomBuildExt
        libraries=[])
    package_dir = {'': '/opendp-bindings/python/'}

elif OPENDP_BUILD == 'static':
    opendp = Extension(
        name='_opendp_ffi',
        sources=[swig_interface_path],
        language='c++',
        swig_opts=[
            "-outdir", os.path.abspath(os.path.join(os.path.dirname(__file__), 'opendp')),
            "-module", "opendp_ffi",
            '-c++',
        ],
        include_dirs=[header_dir],
        extra_objects=[_get_rust_output_path()],
        libraries=[])

else:
    opendp = Extension(
        # this will be the name of the package to import in opendp_ffi.py
        name='_opendp_ffi',
        sources=[swig_interface_path, '/Users/michael/openDP/openDP/rust/opendp-ffi/include/libopendp_wrap.cpp'],
        language='c++',
        swig_opts=[
            "-outdir", os.path.abspath(os.path.join(os.path.dirname(__file__), 'opendp')),
            # CAUTION: this should be the same name as in the module name in libopendp.i
            "-module", "opendp_ffi",
            "-c++",
        ],
        include_dirs=[header_dir],
        # CAUTION: This should be the same as `opendp-ffi/cargo.toml:package.name`, but with an underscore
        libraries=['opendp_ffi'])

setup(
    cmdclass=cmdclass,
    include_package_data=True,
    platforms='any',
    install_requires=[
        'cffi>=1.6.0',
    ],
    setup_requires=[
        'cffi>=1.6.0'
    ],
    ext_modules=[opendp],
    py_modules=['libopendp'],
    package_dir=package_dir)
