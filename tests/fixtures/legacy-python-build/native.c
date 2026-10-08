#define PY_SSIZE_T_CLEAN
#include <Python.h>

static PyObject *versions(PyObject *self, PyObject *args) {
    return Py_BuildValue("(ss)", PY_VERSION, Py_GetVersion());
}

static PyMethodDef methods[] = {
    {"versions", versions, METH_NOARGS, "Return compile-time and runtime Python versions."},
    {NULL, NULL, 0, NULL}
};

static struct PyModuleDef module = {
    PyModuleDef_HEAD_INIT, "_native", NULL, -1, methods
};

PyMODINIT_FUNC PyInit__native(void) {
    return PyModule_Create(&module);
}
