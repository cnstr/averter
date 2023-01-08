FROM rust:1.66 as builder
WORKDIR /app

ADD . .
RUN cargo build --release -p averter

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/averter /
CMD ["./averter"]

EXPOSE 3000
ARG BUILD_DATE
ARG GIT_STATE
ARG VERSION

LABEL org.opencontainers.image.created=${BUILD_DATE}
LABEL org.opencontainers.image.authors="Aarnav Tale <aarnav@tale.me>"
LABEL org.opencontainers.image.source="https://github.com/cnstr/averter"
LABEL org.opencontainers.image.version=${VERSION}
LABEL org.opencontainers.image.revision=${GIT_STATE}
LABEL org.opencontainers.image.vendor="Aarnav Tale"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.ref.name="tale.me/canister/averter"
LABEL org.opencontainers.image.title="Canister Averter"
LABEL org.opencontainers.image.description="Redirects Canister API v1 requests to Canister API v2."
LABEL org.opencontainers.image.base.name="gcr.io/distroless/cc"
