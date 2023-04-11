FROM alpine:latest as build 

ENV LANG zh_CN.UTF-8
ENV LC_ALL zh_CN.UTF-8
ENV LANGUAGE zh_CN.UTF-8
ENV TZ Asia/Shanghai
ENV TERM xterm-256color

RUN apk update && apk add curl
 
RUN curl https://sh.rustup.rs -sSf \
  | sh -s -- -y --no-modify-path --default-toolchain nightly

RUN apk add --no-cache gcc g++ git bash cmake make pkgconfig openssl-dev libc6-compat

SHELL ["/bin/bash","-c"]

WORKDIR /app

ADD sh/jpegxl-rs.sh sh/jpegxl-rs.sh

RUN ./sh/jpegxl-rs.sh

ADD src src
ADD Cargo.toml .
ADD dist.sh .

RUN ./dist.sh

ADD sh sh

RUN ./sh/cpso.sh && ./sh/upx.sh

FROM scratch

ENV RUST_LOG=debug,supervisor=warn,hyper=warn,rustls=warn,quinn_udp=warn
COPY --from=build /app/target/app .

ENV LD_LIBRARY_PATH=/lib
COPY --from=build /so/ /lib/

ENV RUST_BACKTRACE=short

CMD ["/app"]

