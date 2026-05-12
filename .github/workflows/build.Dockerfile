FROM debian:latest AS builder
COPY .out /out
WORKDIR /work
RUN mkdir -p /work
RUN if [ "$(arch)" = "x86_64" ]; then \
        mv /out/binary-amd64 /work/binary; \
    else \
        mv /out/binary-arm64 /work/binary; \
    fi
FROM gcr.io/distroless/cc-debian12
WORKDIR /work
COPY --from=builder /work/binary /work/binary
CMD ["/work/binary"]