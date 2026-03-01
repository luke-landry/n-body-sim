#include <stdio.h>

extern "C" __global__ void hello_gpu() { printf("Hello World from GPU!\n"); }
