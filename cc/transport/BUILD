#
# Copyright 2023 The Project Oak Authors
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#

package(
    default_visibility = ["//visibility:public"],
    licenses = ["notice"],
)

cc_library(
    name = "transport",
    hdrs = ["transport.h"],
    deps = [
        "@com_google_absl//absl/status:statusor",
    ],
)

cc_library(
    name = "grpc_streaming_transport",
    srcs = ["grpc_streaming_transport.cc"],
    hdrs = ["grpc_streaming_transport.h"],
    deps = [
        "//oak_remote_attestation/proto/v1:messages_cc_proto",
        "//oak_remote_attestation/proto/v1:service_streaming_cc_proto",
    ],
)
