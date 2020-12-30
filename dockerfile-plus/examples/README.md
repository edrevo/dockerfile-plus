# Examples

In this folder you can find some example Dockerfile syntax extensions.

## Noop

This is the most basic example. It just adds a new instruction `NOOP` which does nothing (i.e. it is ignored). With this extension, the following Dockerfile would success fully compile:

```dockerfile
# syntax = edrevo/noop-dockerfile

NOOP

FROM alpine

NOOP

WORKDIR /

RUN echo "Hello World"
```