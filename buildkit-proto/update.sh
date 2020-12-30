#!/bin/sh
set -e

export BUILDKIT_VERSION="v0.7.2"

curl "https://raw.githubusercontent.com/moby/buildkit/$BUILDKIT_VERSION/api/types/worker.proto" > proto/github.com/moby/buildkit/api/types/worker.proto
curl "https://raw.githubusercontent.com/moby/buildkit/$BUILDKIT_VERSION/frontend/gateway/pb/gateway.proto" > proto/github.com/moby/buildkit/frontend/gateway/pb/gateway.proto
curl "https://raw.githubusercontent.com/moby/buildkit/$BUILDKIT_VERSION/solver/pb/ops.proto" > proto/github.com/moby/buildkit/solver/pb/ops.proto
curl "https://raw.githubusercontent.com/moby/buildkit/$BUILDKIT_VERSION/util/apicaps/pb/caps.proto" > proto/github.com/moby/buildkit/util/apicaps/pb/caps.proto

curl "https://raw.githubusercontent.com/googleapis/googleapis/master/google/rpc/status.proto" > proto/github.com/gogo/googleapis/google/rpc/status.proto
curl "https://raw.githubusercontent.com/gogo/protobuf/v1.2.1/gogoproto/gogo.proto" > proto/github.com/gogo/protobuf/gogoproto/gogo.proto
curl "https://raw.githubusercontent.com/tonistiigi/fsutil/master/types/stat.proto" > proto/github.com/tonistiigi/fsutil/types/stat.proto
