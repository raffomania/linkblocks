FROM debian:testing-slim
COPY --chmod=755 linkblocks /app/
ENTRYPOINT ["/app/linkblocks", "start"]
