FROM --platform=$BUILDPLATFORM public.ecr.aws/x1i4i6b9/rust-musl-cross-builder AS rust_builder
COPY ./ /src

# Conditional compilation based on the target architecture
ARG TARGETARCH
RUN case ${TARGETARCH} in \
    arm64) \
    echo "Compiling for ARM64" && \
    cargo install --path /src --target=aarch64-unknown-linux-musl ;; \
    amd64) \
    echo "Compiling for AMD64" && \
    cargo install --path /src --target=x86_64-unknown-linux-musl ;; \
    *) \
    echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
    esac

FROM public.ecr.aws/docker/library/alpine:3.20.3
RUN apk add chromium
ENV CHROMIUM_USER_FLAGS="--disable-gpu --headless --no-sandbox --remote-debugging-address=0.0.0.0 --remote-debugging-port=9222 --user-data-dir=/data"
COPY --from=rust_builder /usr/local/cargo/bin/MDHBot /usr/local/bin/MDHBot
WORKDIR /storage
ENTRYPOINT ["/usr/local/bin/MDHBot"]
