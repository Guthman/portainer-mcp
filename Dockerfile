FROM gcr.io/distroless/static-debian12:nonroot

ARG TARGETARCH

COPY portainer-stacks-${TARGETARCH} /usr/local/bin/portainer-stacks

USER 65532:65532

ENTRYPOINT ["/usr/local/bin/portainer-stacks"]
