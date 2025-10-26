# SPDX-FileCopyrightText: 2020 - 2022 Armin Becher <becherarmin@methodpark.de>
#
# SPDX-License-Identifier: Unlicense

FROM ubuntu:24.04

RUN apt-get update; apt-get install -y ca-certificates; update-ca-certificates
RUN apt-get install git -y
ADD ./target/release/mlc /bin/mlc
RUN chmod +x /bin/mlc
RUN PATH=$PATH:/bin/mlc
